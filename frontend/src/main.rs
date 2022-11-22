use gloo_net::http::Request;
use web_sys::{Event, FileList, FormData, HtmlInputElement};
use yew::html::TargetCast;
use yew::{html, Component, Context, Html};

struct FileDetails {
    data: String,
}

pub enum Msg {
    ImageProcessed(String),
    File(web_sys::File),
    None,
}

pub struct App {
    file: Option<FileDetails>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { file: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ImageProcessed(data) => {
                self.file = Some(FileDetails { data });
                true
            }
            Msg::File(file) => {
                let scope = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let a = FormData::new().unwrap();
                    a.append_with_blob("bytes", &file).unwrap();
                    let resp = Request::post("http://127.0.0.1:3000")
                        .body(a)
                        .send()
                        .await
                        .unwrap();

                    scope.send_message(Msg::ImageProcessed(resp.text().await.unwrap()));
                });
                false
            }
            Msg::None => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div id="wrapper">
                <p id="title">{ "Create Composite Image" }</p>
                <input
                    id="file-upload"
                    type="file"
                    accept="image/*"
                    multiple={false}
                    onchange={ctx.link().callback(move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Self::upload_files(input.files())
                    })}
                />
                <div id="area">
                    { for self.file.iter().map(Self::view_file) }
                </div>
            </div>
        }
    }
}

impl App {
    fn view_file(file: &FileDetails) -> Html {
        html! {
            <div class="tile">
                <div class="media">
                    <img alt="uploaded image" height="800px" src={format!("data:image/png;base64,{}", &file.data)} />
                </div>
            </div>
        }
    }

    fn upload_files(files: Option<FileList>) -> Msg {
        if let Some(files) = files {
            let file = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .next()
                .map(|v| web_sys::File::from(v.unwrap()))
                .unwrap();

            Msg::File(file)
        } else {
            Msg::None
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
