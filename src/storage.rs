use rustbreak::{deser::Ron, error::RustbreakError as DbError, FileDatabase};
use serde::{Deserialize, Serialize};

type Item = String;
pub type Items = Vec<Item>;
pub type Categories = Vec<Category>;

const ITEM_DB_PATH: &str = "data/items.ron";
const CATEGORY_DB_PATH: &str = "data/categories.ron";

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
        let res = self.0.write(task)?;
        self.0.save()?;
        Ok(res)
    }

    pub fn add_item(&self, item: Item) -> Result<(), DbError> {
        self.write(|items| {
            if !items.contains(&item) {
                items.push(item)
            }
        })
    }
}
pub struct CategoryDB(FileDatabase<Categories, Ron>);

impl CategoryDB {
    pub fn init() -> Self {
        FileDatabase::load_from_path(CATEGORY_DB_PATH)
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
        let res = self.0.write(task)?;
        self.0.save()?;
        Ok(res)
    }

    pub fn add_category(&self, cat: Category) -> Result<(), DbError> {
        self.write(|cats| {
            if !cats.contains(&cat) {
                cats.push(cat)
            }
        })
    }
}
