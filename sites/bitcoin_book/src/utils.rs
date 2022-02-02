macro_rules! include_markdown_content {
    ($file : expr $(,) ?) => {{
        let md_text = include_str!($file);
        let parser = pulldown_cmark::Parser::new(md_text);
        let mut html_text = String::new();
        pulldown_cmark::html::push_html(&mut html_text, parser);
        let div = gloo::utils::document().create_element("div").unwrap();
        div.set_inner_html(html_text.as_str());
        div.set_class_name("content");
        Html::VRef(div.into())
    }};
}
