use eframe::egui;
use image::{ImageBuffer, Rgb};
use anyhow::Result;
use futures_util::StreamExt;
use tokio::sync::mpsc;
use egui::load::SizedTexture; 
use std::time::Duration;

struct App {
    image: Option<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    client: reqwest::Client,
    url: String,
    connected: bool,
    stream_task: Option<tokio::task::JoinHandle<()>>,
    image_rx: mpsc::UnboundedReceiver<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    reconnect_attempts: u32,
    last_successful_frame: std::time::Instant,
}
impl Default for App {
    fn default() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            image: None,
            client: reqwest::Client::new(),
            url: "http://192.168.0.101:4747/video".to_string(),
            connected: false,
            stream_task: None,
            image_rx: rx,
            reconnect_attempts: 0,
            last_successful_frame: std::time::Instant::now(),
        }
    }
}   

impl App {
    async fn update_image(&mut self) -> Result<(), reqwest::Error> {
        println!("Attempting to connect to: {}", self.url);
        
        let response = self.client
            .get(&self.url)
            .header("Accept", "multipart/x-mixed-replace; boundary=--BoundaryString")
            .send()
            .await?;
            
        println!("Response status: {}", response.status());
        println!("Response headers: {:#?}", response.headers());
        
        let bytes = response.bytes().await?;
        println!("Received {} bytes of data", bytes.len());
        
        // Try to find the JPEG data between boundaries
        match image::load_from_memory(&bytes) {
            Ok(img) => {
                println!("Successfully decoded image: {}x{}", img.width(), img.height());
                self.image = Some(img.to_rgb8());
                self.connected = true;
            }
            Err(e) => {
                eprintln!("Failed to decode image data: {}", e);
                // Print first few bytes to help diagnose format issues
                if !bytes.is_empty() {
                    println!("First 16 bytes: {:?}", &bytes[..bytes.len().min(16)]);
                }
                self.connected = false;
            }
        }
        Ok(())
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.stream_task.is_none() {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .tcp_keepalive(Duration::from_secs(30))
                .tcp_nodelay(true)
                .build()
                .unwrap();
            
            let url = self.url.clone();
            let (tx, rx) = mpsc::unbounded_channel();
            self.image_rx = rx;
            
            self.stream_task = Some(tokio::spawn(async move {
                let mut backoff = Duration::from_millis(100);
                let max_backoff = Duration::from_secs(5);
                
                loop {
                    println!("Attempting to connect to stream...");
                    let response = match client.get(&url)
                        .header("Accept", "multipart/x-mixed-replace; boundary=--BoundaryString")
                        .header("Connection", "keep-alive")
                        .header("Keep-Alive", "timeout=300")
                        .header("Cache-Control", "no-cache")
                        .header("Pragma", "no-cache")
                        .send()
                        .await {
                            Ok(resp) => {
                                if !resp.status().is_success() {
                                    eprintln!("Server returned error: {}", resp.status());
                                    tokio::time::sleep(backoff).await;
                                    backoff = std::cmp::min(backoff * 2, max_backoff);
                                    continue;
                                }
                                backoff = Duration::from_millis(100); // Reset backoff on success
                                resp
                            },
                            Err(e) => {
                                eprintln!("Connection failed: {}", e);
                                tokio::time::sleep(backoff).await;
                                backoff = std::cmp::min(backoff * 2, max_backoff);
                                continue;
                            }
                        };

                    let mut stream = response.bytes_stream();
                    let mut buffer = Vec::with_capacity(65536);
                    let mut last_frame_time = std::time::Instant::now();
                    let frame_timeout = Duration::from_secs(5);

                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                if chunk.is_empty() {
                                    continue;
                                }
                                
                                buffer.extend_from_slice(&chunk);
                                
                                // Process all complete frames in the buffer
                                while let Some(start) = buffer.windows(2).position(|w| w == [0xFF, 0xD8]) {
                                    if let Some(end) = buffer[start..].windows(2).position(|w| w == [0xFF, 0xD9]) {
                                        let end = start + end + 2;
                                        let frame = &buffer[start..end];
                                        
                                        if let Ok(img) = image::load_from_memory(frame) {
                                            last_frame_time = std::time::Instant::now();
                                            let _ = tx.send(img.to_rgb8());
                                        }
                                        
                                        buffer.drain(..end);
                                    } else {
                                        break;
                                    }
                                }
                                
                                // Clear buffer if it gets too large
                                if buffer.len() > 1_048_576 { // 1MB max
                                    buffer.clear();
                                }

                                // Check for frame timeout
                                if last_frame_time.elapsed() > frame_timeout {
                                    println!("No frames received for {:?}, reconnecting...", frame_timeout);
                                    break;
                                }
                            },
                            Err(e) => {
                                eprintln!("Stream error: {}", e);
                                break;
                            }
                        }
                    }
                    
                    println!("Stream ended, reconnecting...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }));
        }

        // Monitor connection health
        if let Some(img) = &self.image {
            self.last_successful_frame = std::time::Instant::now();
        } else if self.last_successful_frame.elapsed() > Duration::from_secs(10) {
            // Reset stream task if no frames for 10 seconds
            if let Some(task) = self.stream_task.take() {
                task.abort();
            }
        }

        // Drain old frames, keep only the latest
        let mut latest_image = None;
        while let Ok(new_image) = self.image_rx.try_recv() {
            latest_image = Some(new_image);
        }
        
        if let Some(img) = latest_image {
            self.image = Some(img);
            self.connected = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(img) = &self.image {
                let size = egui::Vec2::new(img.width() as f32, img.height() as f32);
                let texture = ui.ctx().load_texture(
                    "camera_frame",
                    egui::ColorImage::from_rgb(
                        [img.width() as _, img.height() as _],
                        img.as_raw()
                    ),
                    Default::default()
                );
                let sized_texture = SizedTexture::new(texture.id(), size);
                ui.image(sized_texture);
            } else {
                ui.label("Waiting for camera stream...");
            }
        });

        ctx.request_repaint();
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}

fn main() -> Result<(), eframe::Error> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(3 * 1024 * 1024)  // 3MB stack
        .build()
        .unwrap();
    let _guard = rt.enter();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_resizable(true),
        vsync: false,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    
    eframe::run_native(
        "DroidCam Virtual Camera",
        options,
        Box::new(|_cc| Box::new(App::default()))
    )
}
