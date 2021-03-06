use dodrio::bumpalo::{self, Bump};
use dodrio::{Node, Render, Vdom};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Window;

// Triangle container defining target size.
struct Container {
    size: f64,
}

impl Container {
    // Construct a new container.
    pub fn new() -> Container {
        Container { size: 25.0 }
    }

    // Generate the container style to fluctuate triangle width.
    fn container_transform<'a, 'bump>(&'a self, bump: &'bump Bump, elapsed: f64) -> &'bump str
    where
        'a: 'bump,
    {
        let t = elapsed % 10.0;
        let scale = 0.45 + (if t > 5.0 { 10.0 - t } else { t }) / 40.0;
        let transform = bumpalo::format!(
            in bump,
            "transform: scaleX({}) scaleY(0.7) translateZ(0.1px)",
            scale
        );
        transform.into_bump_str()
    }

    // Generate the dot's position on the grid.
    fn dot_style<'a, 'bump>(&'a self, bump: &'bump Bump, x: f64, y: f64) -> &'bump str
    where
        'a: 'bump,
    {
        let s = self.size * 1.3;
        let radius = s / 2.0;
        let styles = bumpalo::format!(
            in bump,
            r#"
                width: {}px;
                height: {}px;
                left: {}px;
                top: {}px;
                border-radius: {}px;
                line-height: {}px;
            "#,
            s, s, x, y, radius, s
        );
        styles.into_bump_str()
    }

    // Create a dot node.
    fn dot<'a, 'bump>(&'a self, bump: &'bump Bump, x: f64, y: f64, content: u32) -> Node<'bump>
    where
        'a: 'bump,
    {
        use dodrio::builder::{div, text};

        div(bump)
            .attr("class", "dot")
            .attr("style", self.dot_style(bump, x, y))
            .child(text(
                bumpalo::format!(in bump, "{}", content).into_bump_str(),
            ))
            .finish()
    }

    // Create a flattened vector of dot nodes (arranged in recursive triangles)
    fn triangle<'a, 'bump>(
        &'a self,
        bump: &'bump Bump,
        x: f64,
        y: f64,
        s: f64,
        content: u32,
        children: &mut bumpalo::collections::Vec<Node<'bump>>,
    ) where
        'a: 'bump,
    {
        if s <= self.size {
            children.push(self.dot(bump, x, y, content));
            return;
        }

        let s = s / 2.0;

        // Top of triangle
        self.triangle(bump, x, y - (s / 2.0), s, content, children);
        // Bottom-left of triangle
        self.triangle(bump, x - s, y + (s / 2.0), s, content, children);
        // Bottom-right of triangle
        self.triangle(bump, x + s, y + (s / 2.0), s, content, children);
    }
}

impl Render for Container {
    fn render<'a, 'bump>(&'a self, bump: &'bump Bump) -> Node<'bump>
    where
        'a: 'bump,
    {
        use dodrio::builder::div;

        let elapsed = web_sys::window()
            .unwrap_throw()
            .performance()
            .unwrap_throw()
            .now()
            / 1000.0;

        let modulus = elapsed as u32 % 10;

        let mut children = bumpalo::collections::Vec::new_in(bump);
        self.triangle(bump, 0.0, 0.0, 1000.0, modulus, &mut children);

        div(bump)
            .attr("class", "container")
            .attr("style", self.container_transform(bump, elapsed))
            .children(children)
            .finish()
    }
}

// Kick off a loop that keeps re-rendering on every animation frame.
fn animate(window: Window, vdom: Vdom) {
    let rc: Rc<RefCell<Option<Closure<FnMut()>>>> = Rc::new(RefCell::new(None));
    let rc2 = rc.clone();
    let window2 = window.clone();
    let weak = vdom.weak();
    let f = Closure::wrap(Box::new(move || {
        weak.schedule_render();
        window
            .request_animation_frame(
                rc.borrow()
                    .as_ref()
                    .unwrap_throw()
                    .as_ref()
                    .unchecked_ref::<js_sys::Function>(),
            )
            .unwrap_throw();
    }) as Box<FnMut()>);
    window2
        .request_animation_frame(f.as_ref().unchecked_ref::<js_sys::Function>())
        .unwrap_throw();
    *rc2.borrow_mut() = Some(f);

    // Run the virtual DOM and its listeners forever.
    vdom.forget();
}

#[wasm_bindgen(start)]
pub fn run() {
    // Set up the panic hook
    console_error_panic_hook::set_once();

    // Get the scene element to render within
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let scene = document.get_element_by_id("scene").unwrap();

    // Construct a new Container component.
    let container = Container::new();

    // Mount our container component to the scene div.
    let vdom = Vdom::new(&scene, container);

    // Kick off animation loop
    animate(window, vdom);
}
