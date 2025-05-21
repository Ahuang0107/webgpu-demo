use parking_lot::Mutex;
use std::future::Future;
use std::rc::Rc;
use std::sync::Arc;
use wgpu::WasmNotSend;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{
    DeviceEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

pub fn run<T: App + 'static>(title: &'static str) -> Result<(), impl std::error::Error> {
    init_logger();

    let events_loop = EventLoop::new().unwrap();
    let mut app = AppHandler::<T>::new(title);
    events_loop.run_app(&mut app)
}

pub trait App {
    #[allow(opaque_hidden_inferred_bound)]
    fn new(window: Arc<Window>) -> impl Future<Output = Self> + WasmNotSend;

    /// 记录窗口大小已发生变化
    ///
    /// # NOTE:
    /// 当缩放浏览器窗口时, 窗口大小会以高于渲染帧率的频率发生变化，
    /// 如果窗口 size 发生变化就立即调整 surface 大小, 会导致缩放浏览器窗口大小时渲染画面闪烁。
    fn set_window_resized(&mut self, new_size: PhysicalSize<u32>);
    /// 获取窗口大小    
    fn get_size(&self) -> PhysicalSize<u32>;

    /// 键盘事件
    fn keyboard_input(&mut self, _event: &KeyEvent) -> bool {
        false
    }

    fn mouse_click(&mut self, _state: ElementState, _button: MouseButton) -> bool {
        false
    }

    fn mouse_wheel(&mut self, _delta: MouseScrollDelta, _phase: TouchPhase) -> bool {
        false
    }

    fn cursor_move(&mut self, _position: PhysicalPosition<f64>) -> bool {
        false
    }

    /// 鼠标移动/触摸事件
    fn device_input(&mut self, _event: &DeviceEvent) -> bool {
        false
    }

    /// 更新渲染数据
    fn update(&mut self, _dt: instant::Duration) {}

    /// 提交渲染
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
}

pub struct AppHandler<T: App> {
    window: Option<Arc<Window>>,
    title: &'static str,
    app: Rc<Mutex<Option<T>>>,
    /// 错失的窗口大小变化
    ///
    /// # NOTE
    /// 在 web 端，app 的初始化是异步的，当收到 resized 事件时，初始化可能还没有完成从而错过窗口 resized 事件，
    /// 当 app 初始化完成后会调用 `set_window_resized` 方法来补上错失的窗口大小变化事件。
    #[allow(dead_code)]
    missed_resize: Rc<Mutex<Option<PhysicalSize<u32>>>>,
    /// 错失的请求重绘事件
    ///
    /// # NOTE
    /// 在 web 端，app 的初始化是异步的，当收到 redraw 事件时，初始化可能还没有完成从而错过请求重绘事件，
    /// 当 app 初始化完成后会调用 `request_redraw` 方法来补上错失的请求重绘事件, 启用 requestAnimationFrame 帧循环。
    #[allow(dead_code)]
    missed_request_redraw: Rc<Mutex<bool>>,
    /// 上次执行渲染的时间
    last_render_time: instant::Instant,
}

impl<T: App> AppHandler<T> {
    pub fn new(title: &'static str) -> AppHandler<T> {
        AppHandler {
            title,
            window: None,
            app: Rc::new(Mutex::new(None)),
            missed_resize: Rc::new(Mutex::new(None)),
            missed_request_redraw: Rc::new(Mutex::new(false)),
            last_render_time: instant::Instant::now(),
        }
    }

    /// 配置窗口
    fn config_window(&mut self) {
        let window = self.window.as_mut().unwrap();
        window.set_title(self.title);
        window.set_cursor_visible(false);

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;

            let canvas = window.canvas().unwrap();

            // 将 canvas 添加到当前网页中
            web_sys::window()
                .and_then(|win| win.document())
                .map(|doc| {
                    doc.body().map(|body| body.append_child(canvas.as_ref()));
                })
                .expect("无法将 canvas 添加到当前网页中");

            // 确保画布可以获得焦点
            // https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/tabindex
            canvas.set_tab_index(0);

            // 设置画布获得焦点时不显示高亮轮廓
            let style = canvas.style();
            style.set_property("outline", "none").unwrap();
            style.set_property("width", "800px").unwrap();
            style.set_property("height", "600px").unwrap();
            canvas.focus().expect("画布无法获取焦点");
        }
    }

    /// 在提交渲染之前通知窗口系统。
    fn pre_present_notify(&self) {
        if let Some(window) = self.window.as_ref() {
            window.pre_present_notify();
        }
    }

    /// 请求重绘
    fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

impl<T: App + 'static> ApplicationHandler for AppHandler<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("winit application resumed!");
        if self.app.as_ref().lock().is_some() {
            return;
        }

        self.last_render_time = instant::Instant::now();

        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        self.config_window();

        #[cfg(target_arch = "wasm32")]
        {
            let app = self.app.clone();
            let missed_resize = self.missed_resize.clone();
            let missed_request_redraw = self.missed_request_redraw.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let window_cloned = window.clone();

                // NOTE 这里需要注意，必须先执行异步操作创建出 inner_app
                //  然后获取 app 的锁进行更新
                //  如果顺序倒转，会引发死锁，这也是之前打包遇到 Parking not supported on this platform 报错的原因
                let inner_app = T::new(window).await;
                let mut app = app.lock();
                *app = Some(inner_app);

                if let Some(resize) = *missed_resize.lock() {
                    app.as_mut().unwrap().set_window_resized(resize);
                }

                if *missed_request_redraw.lock() {
                    window_cloned.request_redraw();
                }
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let app = pollster::block_on(T::new(window));
            self.app.lock().replace(app);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let mut app = self.app.lock();

        if app.as_ref().is_none() {
            // 如果 app 还没有初始化完成，则记录错失的窗口事件
            match event {
                WindowEvent::Resized(physical_size) => {
                    if physical_size.width > 0 && physical_size.height > 0 {
                        let mut missed_resize = self.missed_resize.lock();
                        *missed_resize = Some(physical_size);
                    }
                }
                WindowEvent::RedrawRequested => {
                    let mut missed_request_redraw = self.missed_request_redraw.lock();
                    *missed_request_redraw = true;
                }
                _ => (),
            }
            return;
        }

        let app = app.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if physical_size.width == 0 || physical_size.height == 0 {
                    // 在 app 内部控制窗口最小化时只更新数据不渲染画面
                    log::info!("Window minimized!");
                } else {
                    log::info!("Window resized: {:?}", physical_size);
                }
                // web 的 surface 最大只支持 2048*2048
                #[cfg(target_arch = "wasm32")]
                let physical_size = PhysicalSize::new(
                    physical_size.width.min(2048),
                    physical_size.height.min(2048),
                );
                log::info!("Window resized(fixed): {:?}", physical_size);

                app.set_window_resized(physical_size);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // 键盘事件
                let _ = app.keyboard_input(&event);
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                // 鼠标滚轮事件
                let _ = app.mouse_wheel(delta, phase);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                // 鼠标点击事件
                let _ = app.mouse_click(state, button);
            }
            WindowEvent::CursorMoved { position, .. } => {
                // 鼠标移动事件
                let _ = app.cursor_move(position);
            }
            WindowEvent::RedrawRequested => {
                // surface 重绘事件
                let now = instant::Instant::now();
                let delta = now - self.last_render_time;
                self.last_render_time = now;

                app.update(delta);

                self.pre_present_notify();

                match app.render() {
                    Ok(_) => {}
                    // 当展示平面的上下文丢失，就需重新配置
                    Err(wgpu::SurfaceError::Lost) => log::error!("Surface is lost"),
                    // 所有其他错误（过期、超时等）应在下一帧解决
                    Err(e) => log::error!("{e:?}"),
                }

                // 除非我们手动请求，RedrawRequested 将只会触发一次。
                self.request_redraw();
            }
            _ => (),
        }
    }
}

fn init_logger() {
    #[cfg(target_arch = "wasm32")]
    {
        // 在 web 上，我们使用 fern，因为 console_log 没有按模块级别过滤功能。
        fern::Dispatch::new()
            .level(log::LevelFilter::Info)
            .level_for("wgpu_core", log::LevelFilter::Info)
            .level_for("wgpu_hal", log::LevelFilter::Error)
            .level_for("naga", log::LevelFilter::Error)
            .chain(fern::Output::call(console_log::log))
            .apply()
            .unwrap();
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // parse_default_env 会读取 RUST_LOG 环境变量，并在这些默认过滤器之上应用它。
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .filter_module("wgpu_core", log::LevelFilter::Info)
            .filter_module("wgpu_hal", log::LevelFilter::Error)
            .filter_module("naga", log::LevelFilter::Error)
            .parse_default_env()
            .init();
    }
}
