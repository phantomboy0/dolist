use colored::Colorize;
use rusqlite::{named_params, params, Connection, Result};
use tabled::Tabled;
use tabled;

#[derive(Tabled)]
#[allow(non_snake_case)]
pub struct TodoItem {
    pub ID: i32,
    pub Name: String,
    pub Description: String,
    #[tabled(display = "format_status")]
    pub Status: bool,
    pub CreatedAt: String,
}


fn format_status(status: &bool) -> String {
    if *status {
        "Done".green().to_string()
    } else {
        "NotDone".red().to_string()
    }
}

pub struct Db {
    conn: Connection,
}

impl Db {
   pub fn new() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open("dolist.db")?;
         conn.execute(
        "CREATE TABLE IF NOT EXISTS dolist (\
        id INTEGER PRIMARY KEY AUTOINCREMENT,\
        name TEXT NOT NULL,\
        description TEXT NOT NULL,\
        done INTEGER NOT NULL,\
        created TEXT NOT NULL)",())?;

      Ok(Db { conn })
   }

  pub fn add_item(&self,name: &str,description: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute("INSERT INTO dolist \
        (name, description, done, created) \
        VALUES (?, ?, 0, datetime('now'))", (name,description))?;

        Ok(())
  }

  pub fn show_items(&self,page: &u32, limit: &u32) -> Result<Vec<TodoItem>,rusqlite::Error> { 
      let offset = (page - 1) * limit;
      let mut stmt = self.conn.prepare("SELECT * FROM dolist ORDER BY id DESC Limit :limit OFFSET :offset")?;
      let result = stmt.query_map(params![limit, offset], |row| {
          Ok(TodoItem {
              ID: row.get(0)?,
              Name: row.get(1)?,
              Description: row.get(2)?,
              Status: row.get(3)?,
              CreatedAt: row.get(4)?,
          })
      })?;

      let mut items = vec![];
      for item in result {
          items.push(item?);
      }

      Ok(items)
  }

    pub fn show_all_items(&self) -> Result<Vec<TodoItem>,rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT * FROM dolist ORDER BY id DESC")?;
        let result = stmt.query_map([], |row| {
            Ok(TodoItem {
                ID: row.get(0)?,
                Name: row.get(1)?,
                Description: row.get(2)?,
                Status: row.get(3)?,
                CreatedAt: row.get(4)?,
            })
        })?;

        let mut items = vec![];
        for item in result {
            items.push(item?);
        }

        Ok(items)
    }
    
    pub fn get_total_number_of_items(&self) -> Result<u32, rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM dolist")?;
        let result = stmt.query_map([], |row| {
            row.get(0)
        })?;
        let mut count:u32 = 0;
        for item in result {
             count = item.unwrap_or(0);

        }
        Ok(count)
    }
    
    pub fn set_item_status(&self, id: &u32, status: &bool) -> Result<usize, rusqlite::Error> {
       let effected = self.conn.execute("UPDATE dolist SET done = ? WHERE id=?",(if *status {1} else {0},id))?;
        
        Ok(effected)
    }
    
    pub fn delete_item(&self, id: &u32) -> Result<usize, rusqlite::Error> {
        let effected =  self.conn.execute("DELETE FROM dolist WHERE id=?",[id])?;
        
        Ok(effected)
    }

    pub fn edit_item(&self, id: &u32, name: &str, description: &str) -> Result<usize, rusqlite::Error> {
     let effected = self.conn.execute("UPDATE dolist
        SET
            name = CASE
                WHEN :name = '' THEN name
                ELSE :name
            END,
            description = CASE
                WHEN :description = '' THEN description
                ELSE :description
            END
        WHERE id = :id",named_params! {":id": id,":name": name, ":description": description})?;

        Ok(effected)
    }
}

