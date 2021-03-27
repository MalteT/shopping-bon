use rustbreak::{deser::Ron, error::RustbreakError as DbError, FileDatabase};
use serde::{Deserialize, Serialize};

pub type Items = Vec<String>;
pub type Categories = Vec<Category>;

const ITEM_DB_PATH: &str = "data/items.ron";
const CATEGORY_DB_PATH: &str = "data/categorys.ron";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Category {
    pub name: String,
    pub icon: char,
}

pub struct ItemDB(FileDatabase<Items, Ron>);

impl ItemDB {
    pub fn init() -> Self {
        FileDatabase::load_from_path_or(ITEM_DB_PATH, vec![])
            .map(ItemDB)
            .expect("Failed to initialize item database!")
    }

    pub fn read<T, R>(&self, task: T) -> Result<R, DbError>
    where
        T: FnOnce(&Items) -> R,
    {
        self.0.read(task)
    }

    pub fn write<T, R>(&self, task: T) -> Result<R, DbError>
    where
        T: FnOnce(&mut Items) -> R,
    {
        self.0.write(task)
    }
}
pub struct CategoryDB(FileDatabase<Categories, Ron>);

impl CategoryDB {
    pub fn init() -> Self {
        FileDatabase::load_from_path_or(CATEGORY_DB_PATH, default_categories())
            .map(CategoryDB)
            .expect("Failed to initialize item database!")
    }

    pub fn read<T, R>(&self, task: T) -> Result<R, DbError>
    where
        T: FnOnce(&Categories) -> R,
    {
        self.0.read(task)
    }

    pub fn write<T, R>(&self, task: T) -> Result<R, DbError>
    where
        T: FnOnce(&mut Categories) -> R,
    {
        self.0.write(task)
    }
}

fn default_categories() -> Categories {
    let categories: &[(&'static str, char)] = include!("../data/categories");
    categories
        .iter()
        .map(|(name, icon)| Category {
            name: name.to_string(),
            icon: *icon,
        })
        .collect()
}
