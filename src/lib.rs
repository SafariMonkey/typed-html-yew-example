#![recursion_limit = "1024"]
use failure::Error;
use serde::Deserialize;
use typed_html::dom::Node;
use typed_html::elements::{input, tr};
use typed_html::output::yew::Yew;
use typed_html::{html, text};
use yew::format::json::Json;
use yew::format::nothing::Nothing;
use yew::services::fetch::{FetchTask, Request, Response};
use yew::services::{ConsoleService, FetchService};
use yew::{ChangeData, Component, ComponentLink, Html, Renderable, ShouldRender};

pub struct Model {
    query: Query,
    console: ConsoleService,
    fetch_service: FetchService,
    ft: Option<FetchTask>,
    link: ComponentLink<Model>,
    table: Vec<OrbitTemplate>,
}
#[derive(Deserialize, Debug)]
pub struct QueryResult {
    objects: Vec<OrbitTemplate>,
    page: i32,
    per_page: i32,
    num_results: i32,
}

#[derive(Deserialize, Debug)]
pub struct OrbitTemplate {
    id: String,
    matter: String,
    brand: String,
    language: String,
    medium: String,
    subject: String,
    body: String,
    created_at: String,
    changed_at: String,
    mime_type: String,
}

impl Model {
    fn search(&mut self) -> FetchTask {
        let request = Request::builder()
            .method("GET")
            .uri("http://foo.bar:4848/templates")
            .body(Nothing)
            .unwrap();

        let callback = self.link.send_back(
            move |response: Response<Json<Result<QueryResult, Error>>>| {
                let (meta, Json(result)) = response.into_parts();
                if !meta.status.is_success() {
                    // self.console.log(&format!("non-ok meta: {:?}", meta));
                    return None;
                }
                match result {
                    Ok(body) => Some(Msg::SearchResults(body)),
                    Err(_) => {
                        // self.console.log(&format!("error fetching: {:?}", err));
                        None
                    }
                }
            },
        );
        self.fetch_service.fetch(request, callback)
    }
}

#[derive(Default, Debug)]
struct Query {
    matter: Option<String>,
    language: Option<String>,
    brand: Option<String>,
    medium: Option<String>,
    mime_type: Option<String>,
}

impl Query {
    fn update(&mut self, filter: FilterUpdate) {
        match filter {
            FilterUpdate::Matter(s) => self.matter = s,
            FilterUpdate::Langauge(s) => self.language = s,
            FilterUpdate::Brand(s) => self.brand = s,
            FilterUpdate::Medium(s) => self.medium = s,
            FilterUpdate::MimeType(s) => self.mime_type = s,
        }
    }
}

#[derive(Debug)]
pub enum Msg {
    QueryFilterUpdate(FilterUpdate),
    SearchAction,
    SearchResults(QueryResult),
}

impl From<FilterUpdate> for Msg {
    fn from(filter: FilterUpdate) -> Self {
        Msg::QueryFilterUpdate(filter)
    }
}

#[derive(Debug)]
pub enum FilterUpdate {
    Matter(Option<String>),
    Langauge(Option<String>),
    Brand(Option<String>),
    Medium(Option<String>),
    MimeType(Option<String>),
}

impl Component for Model {
    type Message = Option<Msg>;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model {
            query: Query::default(),
            console: ConsoleService::new(),
            fetch_service: FetchService::new(),
            ft: None,
            link,
            table: Vec::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        if let Some(msg) = msg {
            self.console.log(&format!("{:?}", msg));

            match msg {
                Msg::QueryFilterUpdate(filter) => self.query.update(filter),
                Msg::SearchAction => self.ft = Some(self.search()),
                Msg::SearchResults(result) => {
                    self.console.log(&format!("got result: {:?}", result));
                    self.table = result.objects
                }
            }

            self.console.log(&format!("{:?}", self.query));
            return true;
        }
        false
    }
}

const TABLE_FIELDS: &'static [&'static str] = &[
    "Subject",
    "Brand",
    "Language",
    "Medium",
    "Matter",
    "MIME Type",
    "Created At",
    "Changed At",
    "Body",
    "Actions",
];

fn query_field(
    name: &str,
    filter_factory: impl Fn(Option<String>) -> Option<FilterUpdate> + 'static,
) -> Box<input<Yew<Model>>> {
    html! (
        <input type="text" name=name onchange={move |value| {
            match value {
                ChangeData::Value(val) => {
                    filter_factory(none_if_empty(val)).map(Into::into)
                }
                _ => None,
            }
        }}/>
    : Yew<Model>)
}

fn template_row(template: &OrbitTemplate) -> Box<tr<Yew<Model>>> {
    html! (
        <tr>
            <td> { text!(template.subject.to_owned()) } </td>
            <td> { text!(template.brand.to_owned()) } </td>
            <td> { text!(template.language.to_owned()) } </td>
            <td> { text!(template.medium.to_owned()) } </td>
            <td> { text!(template.matter.to_owned()) } </td>
            <td> { text!(template.mime_type.to_owned()) } </td>
            <td> { text!(template.created_at.to_owned()) } </td>
            <td> { text!(template.changed_at.to_owned()) } </td>
            <td> { text!(template.body.to_owned()) } </td>
        </tr>
    : Yew<Model>)
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        let mut doc = html! (
            <div>
                <div class="filters">
                    <h1>"Search Templates"</h1>

                    { query_field("Matter", move |val| Some(FilterUpdate::Matter(val))) }
                    { query_field("Language", move |val| Some(FilterUpdate::Langauge(val))) }
                    { query_field("Brand", move |val| Some(FilterUpdate::Brand(val))) }
                    { query_field("Medium", move |val| Some(FilterUpdate::Medium(val))) }
                    { query_field("MimeType", move |val| Some(FilterUpdate::MimeType(val))) }

                <button type="button" onclick={|_| Some(Msg::SearchAction)}>"Search"</button>

                </div>
                <section class="table">
                    <table>
                        <thead>
                            <tr>
                                {TABLE_FIELDS.iter().map(|x|
                                    html! {
                                        <th>
                                            { text!(*x) }
                                        </th>
                                    }
                                )}
                            </tr>
                        </thead>
                        <tbody>
                            {self.table.iter().map(template_row)}
                        </tbody>
                    </table>
                </section>
            </div>
        : Yew<Self>);
        Yew::build(doc.vnode())
    }
}
