use chrono::{DateTime, Utc};
use futures::StreamExt;
use relm4::component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender};
use relm4::factory::AsyncFactoryVecDeque;
use relm4::gtk::traits::ButtonExt;
use relm4::prelude::DynamicIndex;
use relm4::{Component, ComponentController, Controller, JoinHandle, RelmWidgetExt, tokio};
use relm4::{
    adw,
    adw::prelude::NavigationPageExt,
    gtk,
    gtk::prelude::{BoxExt, OrientableExt, WidgetExt},
};
use relm4_icons::icon_name;

use done_core::models::status::Status;
use done_core::models::task::Task;
use done_core::service::Service;

use crate::app::components::task_input::TaskInputOutput;
use crate::app::factories::task::{TaskInit, TaskInput, TaskModel, TaskOutput};
use crate::app::models::sidebar_list::SidebarList;
use crate::fl;

use super::task_input::{TaskInputInput, TaskInputModel};
use super::welcome::WelcomeComponent;

pub struct ContentModel {
    task_factory: AsyncFactoryVecDeque<TaskModel>,
    task_entry: Controller<TaskInputModel>,
    welcome: Controller<WelcomeComponent>,
    state: ContentState,
    service: Service,
    parent_list: Option<SidebarList>,
    handle: Option<JoinHandle<()>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContentState {
    Unselected,
    Empty,
    Loading,
    TasksLoaded,
}

#[derive(Debug)]
pub enum ContentInput {
    AddTask(Task),
    RemoveTask(DynamicIndex),
    UpdateTask(Task),
    LoadTask(Task),
    SelectList(SidebarList, Service),
    ServiceDisabled(Service),
    LoadTasks(SidebarList, Service),
    SetState(ContentState),
    ExpandSubTasks(bool),
    CollapseSidebar,
    Clean,
}

#[derive(Debug)]
pub enum ContentOutput {
    CollapseSidebar,
}

#[relm4::component(pub async)]
impl AsyncComponent for ContentModel {
    type CommandOutput = ();
    type Input = ContentInput;
    type Output = ContentOutput;
    type Init = Option<Service>;

    view! {
        #[root]
        adw::ToolbarView {
            #[name = "content_header"]
            add_top_bar = &adw::HeaderBar {
                set_hexpand: true,
                set_css_classes: &["flat"],
                set_show_start_title_buttons: false,
                set_show_end_title_buttons: true,
                #[watch]
                set_title_widget: Some(&gtk::Label::new(
                    Some("Tasks")
                )),
                pack_start: sidebar_button = &gtk::Button {
                    set_icon_name: icon_name::DOCK_LEFT,
                    connect_clicked => ContentInput::CollapseSidebar,
                },
                pack_start = &gtk::Button {
                    set_visible: false,
                    set_tooltip: fl!("search"),
                    set_icon_name: icon_name::LOUPE,
                },
            },
            #[name(overlay)]
            #[wrap(Some)]
            set_content = &adw::ToastOverlay {
                #[wrap(Some)]
                set_child = &gtk::Box {
                    gtk::Box {
                        #[watch]
                        set_visible: model.parent_list.is_none(),
                        append: model.welcome.widget()
                    },
                    adw::Clamp {
                        gtk::Box {
                            #[watch]
                            set_visible: model.parent_list.is_some(),
                            set_orientation: gtk::Orientation::Vertical,
                            #[transition = "Crossfade"]
                            append = match model.state {
                                ContentState::Unselected => {
                                    gtk::Box {
                                        set_vexpand: true,
                                        set_hexpand: true,
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_halign: gtk::Align::Center,
                                        set_valign: gtk::Align::Center,
                                        set_spacing: 10,
                                        gtk::Image {
                                            set_icon_name: Some(icon_name::SONAR),
                                            set_pixel_size: 64,
                                            set_margin_all: 10,
                                        },
                                        gtk::Label {
                                            set_css_classes: &["title-2"],
                                            set_wrap: true,
                                            set_wrap_mode: gtk::pango::WrapMode::Word,
                                            set_justify: gtk::Justification::Center,
                                            #[watch]
                                            set_text: fl!("list-empty"),
                                        },
                                        gtk::Label {
                                            set_css_classes: &["body"],
                                            #[watch]
                                            set_text: fl!("instructions"),
                                            set_wrap: true,
                                            set_wrap_mode: gtk::pango::WrapMode::Word,
                                            set_justify: gtk::Justification::Center,
                                        },
                                    }
                                },
                                ContentState::Loading => {
                                    gtk::CenterBox {
                                        set_orientation: gtk::Orientation::Vertical,
                                        #[name(spinner)]
                                        #[wrap(Some)]
                                        set_center_widget = &gtk::Spinner {
                                            start: ()
                                        }
                                    }
                                },
                                ContentState::TasksLoaded | ContentState::Empty => {
                                    #[name(split_view)]
                                    adw::NavigationView {
                                        add = &adw::NavigationPage {
                                            set_title: "Smart",
                                            set_tag: Some("smart"),
                                            #[wrap(Some)]
                                            set_child = &gtk::Box {
                                                set_orientation: gtk::Orientation::Vertical,
                                                set_margin_all: 10,
                                                gtk::Box {
                                                    #[watch]
                                                    set_orientation: gtk::Orientation::Horizontal,
                                                    gtk::Image {
                                                        #[watch]
                                                        set_visible: model.parent_list.as_ref().unwrap().smart(),
                                                        #[watch]
                                                        set_icon_name: model.parent_list.as_ref().unwrap().icon(),
                                                        set_margin_start: 10,
                                                    },
                                                    gtk::Label {
                                                        #[watch]
                                                        set_visible: !model.parent_list.as_ref().unwrap().smart(),
                                                        #[watch]
                                                        set_text: model.parent_list.as_ref().unwrap().icon().unwrap_or_default(),
                                                        set_margin_start: 10,
                                                    },
                                                    gtk::Label {
                                                        set_css_classes: &["title-3"],
                                                        set_halign: gtk::Align::Start,
                                                        set_margin_start: 10,
                                                        set_margin_end: 10,
                                                        #[watch]
                                                        set_text: model.parent_list.as_ref().unwrap().name().as_str()
                                                    },
                                                },
                                                gtk::Label {
                                                    #[watch]
                                                    set_visible: !model.parent_list.as_ref().unwrap().description().is_empty(),
                                                    set_css_classes: &["title-5"],
                                                    set_halign: gtk::Align::Start,
                                                    set_margin_bottom: 10,
                                                    set_margin_start: 10,
                                                    set_margin_end: 10,
                                                    #[watch]
                                                    set_text: model.parent_list.as_ref().unwrap().description().as_str()
                                                },
                                                #[name(task_container)]
                                                gtk::Stack {
                                                    set_transition_duration: 250,
                                                    set_transition_type: gtk::StackTransitionType::Crossfade,
                                                    if model.task_factory.is_empty() {
                                                        gtk::Box {
                                                            set_vexpand: true,
                                                            set_hexpand: true,
                                                            set_orientation: gtk::Orientation::Vertical,
                                                            set_halign: gtk::Align::Center,
                                                            set_valign: gtk::Align::Center,
                                                            set_spacing: 10,
                                                            gtk::Image {
                                                                set_icon_name: Some(icon_name::CHECK_ROUND_OUTLINE2),
                                                                set_pixel_size: 64,
                                                                set_margin_all: 10,
                                                            },
                                                            gtk::Label {
                                                                set_css_classes: &["title-2"],
                                                                set_wrap: true,
                                                                set_wrap_mode: gtk::pango::WrapMode::Word,
                                                                set_justify: gtk::Justification::Center,
                                                                #[watch]
                                                                set_text: fl!("all-done"),
                                                            },
                                                            gtk::Label {
                                                                set_css_classes: &["body"],
                                                                #[watch]
                                                                set_text: fl!("all-done-instructions"),
                                                                set_wrap: true,
                                                                set_wrap_mode: gtk::pango::WrapMode::Word,
                                                                set_justify: gtk::Justification::Center,
                                                            },
                                                        }
                                                    } else {
                                                        gtk::ScrolledWindow {
                                                            set_visible: !model.task_factory.is_empty(),
                                                            #[watch]
                                                            set_visible: model.state == ContentState::TasksLoaded,
                                                            set_vexpand: true,
                                                            set_hexpand: true,
                                                            #[local_ref]
                                                            list_box -> adw::PreferencesGroup {
                                                                set_css_classes: &["boxed-list"],
                                                                set_valign: gtk::Align::Fill,
                                                                set_margin_all: 5,
                                                            },
                                                        }
                                                    }
                                                },
                                                gtk::Box {
                                                    set_margin_all: 5,
                                                    append: model.task_entry.widget()
                                                }
                                            },
                                        },
                                    }
                                }
                            }
                        }
                    },
                }
            },
        },
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = ContentModel {
            task_factory: AsyncFactoryVecDeque::builder()
                .launch(adw::PreferencesGroup::default())
                .forward(sender.input_sender(), |output| match output {
                    TaskOutput::Remove(index) => ContentInput::RemoveTask(index),
                    TaskOutput::UpdateTask(task) => ContentInput::UpdateTask(task),
                }),
            task_entry: TaskInputModel::builder()
                .launch(SidebarList::default())
                .forward(sender.input_sender(), |message| match message {
                    TaskInputOutput::AddTask(task) => ContentInput::AddTask(task),
                }),
            welcome: WelcomeComponent::builder().launch(()).detach(),
            state: ContentState::Unselected,
            service: Service::Smart,
            parent_list: None,
            handle: None,
        };

        let list_box = model.task_factory.widget();

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ContentInput::Clean => self.state = ContentState::Unselected,
            ContentInput::SetState(state) => self.state = state,
            ContentInput::ExpandSubTasks(expand) => {
                println!("{}", self.task_factory.len());
                for (i, _) in self.task_factory.iter().enumerate() {
                    self.task_factory.send(i, TaskInput::ExpandSubTask(expand))
                }
            }
            ContentInput::CollapseSidebar => sender
                .output(ContentOutput::CollapseSidebar)
                .unwrap_or_default(),
            ContentInput::LoadTask(task) => {
                if let SidebarList::Custom(parent) = &self.parent_list.as_ref().unwrap() {
                    let mut guard = self.task_factory.guard();
                    guard.push_back(TaskInit::new(task, parent.clone()));
                    self.state = ContentState::TasksLoaded;
                }
            }
            ContentInput::AddTask(mut task) => {
                if let SidebarList::Custom(parent) = &self.parent_list.as_ref().unwrap() {
                    task.parent = parent.id.clone();
                    let mut service = self.service.get_service();
                    match service.create_task(task.clone()).await {
                        Ok(_) => {
                            self.task_factory
                                .guard()
                                .push_back(TaskInit::new(task.clone(), parent.clone()));
                            self.state = ContentState::TasksLoaded;
                        }
                        Err(err) => {
                            tracing::error!("An error ocurred: {err}");
                        }
                    }
                }
            }
            ContentInput::RemoveTask(index) => {
                let mut guard = self.task_factory.guard();
                if let Some(task) = guard.get(index.current_index()) {
                    let mut service = self.service.get_service();
                    match service
                        .delete_task(task.task.clone().parent, task.task.clone().id)
                        .await
                    {
                        Ok(_) => {
                            guard.remove(index.current_index());
                        }
                        Err(err) => tracing::error!("An error ocurred: {err}"),
                    }
                }
            }
            ContentInput::UpdateTask(task) => {
                let mut service = self.service.get_service();
                match service.update_task(task).await {
                    Ok(task) => tracing::info!("Task {} successfully saved.", task.id),
                    Err(err) => tracing::error!("An error ocurred: {err}"),
                }
            }
            ContentInput::SelectList(list, service) => {
                self.state = ContentState::Loading;
                if let Some(handle) = &self.handle {
                    handle.abort()
                }
                sender.input(ContentInput::LoadTasks(list, service));
            }
            ContentInput::LoadTasks(list, service) => {
                let mut guard = self.task_factory.guard();
                guard.clear();
                self.service = service;

                let mut service = service.get_service();
                if let Ok(tasks) = service.read_tasks().await {
                    match &list {
                        SidebarList::All => {
                            self.parent_list = Some(SidebarList::All);
                            for task in tasks {
                                guard.push_back(TaskInit::new(
                                    task.clone(),
                                    service.read_list(task.parent).await.unwrap(),
                                ));
                            }
                            self.state = ContentState::TasksLoaded;
                        }
                        SidebarList::Today => {
                            self.parent_list = Some(SidebarList::Today);
                            for task in tasks.iter().filter(|task| {
                                task.today
                                    || task.due_date.is_some()
                                        && task.due_date.unwrap().date_naive()
                                            == Utc::now().date_naive()
                            }) {
                                guard.push_back(TaskInit::new(
                                    task.clone(),
                                    service.read_list(task.parent.clone()).await.unwrap(),
                                ));
                            }
                            self.state = ContentState::TasksLoaded;
                        }
                        SidebarList::Starred => {
                            self.parent_list = Some(SidebarList::Starred);
                            for task in tasks.iter().filter(|task| task.favorite) {
                                guard.push_back(TaskInit::new(
                                    task.clone(),
                                    service.read_list(task.parent.clone()).await.unwrap(),
                                ));
                            }
                            self.state = ContentState::TasksLoaded;
                        }
                        SidebarList::Next7Days => {
                            self.parent_list = Some(SidebarList::Next7Days);
                            for task in tasks.iter().filter(|task: &&Task| {
                                task.due_date.is_some()
                                    && is_within_next_7_days(task.due_date.unwrap())
                            }) {
                                guard.push_back(TaskInit::new(
                                    task.clone(),
                                    service.read_list(task.parent.clone()).await.unwrap(),
                                ));
                            }
                            self.state = ContentState::TasksLoaded;
                        }
                        SidebarList::Done => {
                            self.parent_list = Some(SidebarList::Done);
                            for task in tasks
                                .iter()
                                .filter(|task: &&Task| task.status == Status::Completed)
                            {
                                guard.push_back(TaskInit::new(
                                    task.clone(),
                                    service.read_list(task.parent.clone()).await.unwrap(),
                                ));
                            }
                            self.state = ContentState::TasksLoaded;
                        }
                        SidebarList::Custom(list) => {
                            self.parent_list = Some(SidebarList::Custom(list.clone()));
                            let sender_clone = sender.clone();
                            let list_clone = list.clone();
                            let mut service = self.service.get_service();
                            self.state = ContentState::Loading;
                            if service.stream_support() {
                                self.handle = Some(tokio::spawn(async move {
                                    match service.get_tasks(list_clone.id.clone()).await {
                                        Ok(mut stream) => {
                                            let first = stream.next().await;
                                            if let Some(task) = first {
                                                sender_clone.input(ContentInput::LoadTask(task));
                                                while let Some(task) = stream.next().await {
                                                    sender_clone
                                                        .input(ContentInput::LoadTask(task));
                                                }
                                            } else {
                                                sender_clone.input(ContentInput::SetState(
                                                    ContentState::Empty,
                                                ));
                                            }
                                        }
                                        Err(err) => tracing::error!("{err}"),
                                    }
                                }));
                            } else if let Ok(tasks) =
                                service.read_tasks_from_list(list_clone.id.clone()).await
                            {
                                if tasks.is_empty() {
                                    self.state = ContentState::Empty;
                                } else {
                                    for task in &tasks {
                                        guard.push_back(TaskInit::new(
                                            task.clone(),
                                            service.read_list(task.parent.clone()).await.unwrap(),
                                        ));
                                    }
                                    self.state = ContentState::TasksLoaded;
                                }
                            } else {
                                self.state = ContentState::Empty;
                            }
                        }
                    }
                }

                if guard.is_empty() && self.state != ContentState::Loading {
                    self.state = ContentState::Empty;
                }

                if list.smart() {
                    self.state = ContentState::Unselected;
                }

                self.task_entry
                    .sender()
                    .send(TaskInputInput::SetParentList(
                        self.parent_list.as_ref().unwrap().clone(),
                    ))
                    .unwrap();
            }
            ContentInput::ServiceDisabled(service) => {
                if self.service == service {
                    self.state = ContentState::Unselected;
                }
            }
        }
        self.update_view(widgets, sender)
    }
}

fn is_within_next_7_days(date: DateTime<Utc>) -> bool {
    let now = Utc::now();
    let next_7_days = now + chrono::Duration::days(7);
    date >= now && date <= next_7_days
}
