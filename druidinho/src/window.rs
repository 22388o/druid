use crate::kurbo::{Point, Size};
use crate::piet::Piet;
use crate::widget_host::{WidgetHost, WidgetState};
use crate::{BoxConstraints, EventCtx, LayoutCtx, PaintCtx, Widget};
use druid_shell::{IdleToken, KeyEvent, MouseEvent, Region, TimerToken, WindowHandle};

pub struct Window {
    handle: WindowHandle,
    root_state: WidgetState,
    window_size: Size,
    root: WidgetHost<Box<dyn Widget>>,
}

impl Window {
    fn with_event_ctx<R>(&mut self, f: impl FnOnce(&mut dyn Widget, &mut EventCtx) -> R) -> R {
        //let mut widget_state = WidgetState::default();
        let mut ctx = EventCtx {
            window: &self.handle,
            state: &mut self.root_state,
        };
        f(&mut self.root, &mut ctx)
    }

    pub fn new(handle: WindowHandle, root: Box<dyn Widget>) -> Self {
        Window {
            handle,
            root: WidgetHost::new(root),
            window_size: Size::ZERO,
            root_state: Default::default(),
        }
    }

    pub fn window_connected(&mut self) {
        self.with_event_ctx(|chld, ctx| chld.init(ctx));
    }

    pub fn prepare_paint(&mut self) {
        let mut ctx = LayoutCtx {
            state: &self.root_state,
            window: &self.handle,
        };
        let bc = BoxConstraints::tight(self.window_size.clone());
        self.root.layout(&mut ctx, bc);
        self.root.set_origin(Point::ZERO);
    }

    pub fn paint(&mut self, piet: &mut Piet, _region: &Region) {
        let mut ctx = PaintCtx {
            state: &self.root_state,
            render_ctx: piet,
        };

        self.root.paint(&mut ctx);
    }

    pub fn size_changed(&mut self, new_size: Size) {
        self.window_size = new_size;
        self.handle.invalidate();
    }

    pub fn mouse_down(&mut self, event: &MouseEvent) {
        let event = event.to_owned().into();
        self.with_event_ctx(|chld, ctx| chld.mouse_down(ctx, &event))
    }

    pub fn mouse_up(&mut self, event: &MouseEvent) {
        let event = event.to_owned().into();
        self.with_event_ctx(|chld, ctx| chld.mouse_up(ctx, &event))
    }

    pub fn mouse_move(&mut self, event: &MouseEvent) {
        let event = event.to_owned().into();
        self.with_event_ctx(|chld, ctx| chld.mouse_move(ctx, &event))
    }

    pub fn scroll(&mut self, event: &MouseEvent) {
        let event = event.to_owned().into();
        self.with_event_ctx(|chld, ctx| chld.scroll(ctx, &event))
    }

    pub fn key_down(&mut self, event: KeyEvent) -> bool {
        self.with_event_ctx(|chld, ctx| chld.key_down(ctx, &event));
        false
    }

    pub fn key_up(&mut self, event: KeyEvent) {
        self.with_event_ctx(|chld, ctx| chld.key_up(ctx, &event))
    }

    pub fn timer(&mut self, token: TimerToken) {
        self.with_event_ctx(|chld, ctx| chld.timer(ctx, token))
    }

    pub fn idle(&mut self, _token: IdleToken) {
        //self.with_event_ctx(|chld, ctx| chld.id(ctx, token))
    }
}
