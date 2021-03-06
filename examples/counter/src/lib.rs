use dodrio::bumpalo::{self, Bump};
use dodrio::Render;
use log::*;
use wasm_bindgen::prelude::*;

/// A counter that can be incremented and decremented!
struct Counter {
    count: isize,
}

impl Counter {
    /// Construct a new, zeroed counter.
    fn new() -> Counter {
        Counter { count: 0 }
    }

    /// Increment this counter's count.
    fn increment(&mut self) {
        self.count += 1;
    }

    /// Decrement this counter's count.
    fn decrement(&mut self) {
        self.count -= 1;
    }
}

// The `Render` implementation for `Counter`s displays the current count and has
// buttons to increment and decrement the count.
impl Render for Counter {
    fn render<'a, 'bump>(&'a self, bump: &'bump Bump) -> dodrio::Node<'bump>
    where
        'a: 'bump,
    {
        use dodrio::builder::*;

        // Stringify the count as a bump-allocated string.
        let count = bumpalo::format!(in bump, "{}", self.count);

        div(bump)
            .children([
                button(bump)
                    .on("click", |root, vdom, _event| {
                        // Cast the root render component to a `Counter`, since
                        // we know that's what it is.
                        let counter = root.unwrap_mut::<Counter>();

                        // Increment the counter.
                        counter.increment();

                        // Since the count has updated, we should re-render the
                        // counter on the next animation frame.
                        vdom.schedule_render();
                    })
                    .children([text("+")])
                    .finish(),
                text(count.into_bump_str()),
                button(bump)
                    .on("click", |root, vdom, _event| {
                        // Same as above, but decrementing instead of incrementing.
                        root.unwrap_mut::<Counter>().decrement();
                        vdom.schedule_render();
                    })
                    .children([text("-")])
                    .finish(),
            ])
            .finish()
    }
}

#[wasm_bindgen(start)]
pub fn run() {
    // Initialize debug logging for if/when things go wrong.
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Trace).expect("should initialize logging OK");

    // Get the document's `<body>`.
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    // Construct a new counter component.
    let counter = Counter::new();

    // Mount our counter component to the `<body>`.
    let vdom = dodrio::Vdom::new(&body, counter);

    // Run the virtual DOM and its listeners forever.
    vdom.forget();
}
