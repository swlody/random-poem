use maud::{html, Markup, DOCTYPE};

#[tracing::instrument]
pub fn wrap_body(content: &Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang = "en" {
            head {
                meta charset="utf-8";
                link rel = "stylesheet" href="/static/style.css";
                script {
                    "0"
                }
            }
            body {
                (content)
            }
        }
    }
}
