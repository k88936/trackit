use tabled::{Table, Tabled, settings::Style};

pub fn format_table<T: Tabled>(data: &[T]) -> String {
    Table::new(data).with(Style::rounded()).to_string()
}
