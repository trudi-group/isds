use super::*;

#[function_component(Beyond)]
pub fn beyond() -> Html {
    html! {
        <>
            <header class="section">
                <NavBar />
                <h2 class="title">{ "More places to learn" }</h2>
                { include_markdown_content!("intro.md") }
            </header>
            <main class="section">
                { include_markdown_content!("content.md") }
            </main>
        </>
    }
}

