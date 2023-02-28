use maud::{html, Markup, PreEscaped};
use pulldown_cmark::{Parser, Options, html as cmark_html};

pub fn render_markdown(content: &str) -> Markup {
    let parser = Parser::new_ext(content, Options::empty());
    let mut rendered = String::new();
    cmark_html::push_html(&mut rendered, parser);

    html! { (PreEscaped(rendered)) }
}
