macro_rules! include_markdown_content {
    ($file : expr $(,) ?) => {{
        let md_text = include_str!($file);
        markdown_content!(md_text)
    }};
}

macro_rules! indoc_markdown_content {
    ($raw_md_text : expr $(,) ?) => {{
        let md_text = indoc::indoc!($raw_md_text);
        markdown_content!(md_text)
    }};
}

macro_rules! markdown_content {
    ($md_text : expr $(,) ?) => {{
        let parser = pulldown_cmark::Parser::new($md_text);
        let mut html_text = String::new();
        pulldown_cmark::html::push_html(&mut html_text, parser);
        let div = gloo::utils::document().create_element("div").unwrap();
        div.set_inner_html(html_text.as_str());
        div.set_class_name("content");
        Html::VRef(div.into())
    }};
}
