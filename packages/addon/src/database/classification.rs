use super::*;

#[napi(string_enum)]
#[derive(Debug, Deserialize, Serialize)]
pub enum ItemLayout {
    Default,
    Tile,
    List,
}

#[napi(string_enum)]
#[derive(Debug, Deserialize, Serialize)]
pub enum ItemSort {
    Default,
    Initial,
    OpenNumber,
    LastOpen,
}

#[napi(string_enum)]
#[derive(Debug, Deserialize, Serialize)]
pub enum ItemShowOnly {
    Default,
    File,
    Folder,
}

#[napi]
#[derive(Debug, Deserialize, Serialize)]
pub struct ClassificationData {
    pub icon: Option<String>,
    pub associate_folder_path: Option<String>,
    pub associate_folder_hidden_items: Option<String>,
    pub item_layout: ItemLayout,
    pub item_sort: ItemSort,
    pub item_column_number: Option<i32>,
    pub item_icon_size: Option<i32>,
    pub item_show_only: ItemShowOnly,
    pub fixed: bool,
    pub aggregate_item_count: i32,
    pub exclude_search: bool,
}

#[napi]
#[derive(Debug, Deserialize)]
pub struct Classification {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub type_: i32,
    pub data: String,
    pub shortcut_key: Option<String>,
    pub global_shortcut_key: bool,
    pub order: i32,
}

#[napi]
impl Classification {
    #[napi(constructor)]
    pub fn new() -> Self {
        unimplemented!()
    }

    #[napi]
    pub fn get_child_list(
        &self,
        parent_id: i64,
        data_source: &DataSource,
    ) -> Result<Vec<Classification>> {
        data_source.get_classification(Some(parent_id))
    }
}

impl Default for ClassificationData {
    fn default() -> Self {
        ClassificationData {
            icon: None,
            associate_folder_path: None,
            associate_folder_hidden_items: None,
            item_layout: ItemLayout::Default,
            item_sort: ItemSort::Default,
            item_column_number: None,
            item_icon_size: None,
            item_show_only: ItemShowOnly::Default,
            fixed: false,
            aggregate_item_count: 50,
            exclude_search: false,
        }
    }
}

impl Default for Classification {
    fn default() -> Self {
        Classification {
            id: 0,
            parent_id: None,
            name: None,
            type_: 0,
            data: Default::default(),
            shortcut_key: None,
            global_shortcut_key: false,
            order: 0,
        }
    }
}
