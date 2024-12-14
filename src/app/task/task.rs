use gtk::prelude::*;
use relm4::prelude::*;
#[derive(Debug)]
pub struct Task {
    name: String,
    completed: bool,
}

#[derive(Debug)]
pub enum TaskInput {
    Toggle(bool),
}

#[derive(Debug)]
pub enum TaskOutput {
    Delete(DynamicIndex),
}

#[relm4::factory(pub)]
impl FactoryComponent for Task {
    type Init = String;
    type Input = TaskInput;
    type Output = TaskOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,

            gtk::CheckButton {
                set_active: false,
                set_margin_all: 12,
                connect_toggled[sender] => move |checkbox| {
                    sender.input(TaskInput::Toggle(checkbox.is_active()));
                }
            },

            #[name(label)]
            gtk::Label {
                set_label: &self.name,
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                set_margin_all: 12,
            },

            gtk::Button {
                set_icon_name: "edit-delete",
                set_margin_all: 12,

                connect_clicked[sender, index] => move |_| {
                    sender.output(TaskOutput::Delete(index.clone())).unwrap();
                }
            }
        }
    }

    fn pre_view() {
        let attrs = widgets.label.attributes().unwrap_or_default();
        attrs.change(gtk::pango::AttrInt::new_strikethrough(self.completed));
        widgets.label.set_attributes(Some(&attrs));
    }

    fn init_model(name: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            name,
            completed: false,
        }
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            TaskInput::Toggle(completed) => {
                self.completed = completed;
            }
        }
    }
}
