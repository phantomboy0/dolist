mod repository;
use repository::database::Db;
use clap::{value_parser, Arg, ArgAction, Command};
use colored::Colorize;
use tabled::{Table};
use tabled::settings::{Color, Style};
use tabled::settings::object::Rows;
use tabled::settings::themes::Colorization;

fn main() {
    let matches = Command::new("dolist").
        version("v1.0.0").
        author("phantomboy0").
        about("CLI to do list").
        help_template("Dolist {version} By {author} \n{about} \n\n{usage}\n\n{all-args}").
        //$ dolist new "Buy food" "buy orange and bleach for home"
        subcommand(Command::new("new").
            about("Create a new to-do item").

                arg(Arg::new("name").
                help("Title of the to-do item").
                required(true).
                index(1)).

                arg(Arg::new("description").
                help("Description of this item").
                index(2))).
        
        //$ dolist list 1 -l 15 -a
        subcommand(Command::new("list").
            about("Shows list of todo items").

            arg(Arg::new("page").
                help("Page number of the list").
                value_parser(value_parser!(u32)).
                default_value("1").
                index(1)).

            arg(Arg::new("limit").
                long("limit").
                short('l').
                help("Limit number of items in list per page").
                value_parser(value_parser!(u32)).
                default_value("10")).

            arg(Arg::new("all").
                long("all").
                short('a').
                help("Shows the entire items in the list").
                action(ArgAction::SetTrue))).

        //$ dolist done 1
        subcommand(Command::new("done").
            about("Set a Item's Status as Done").

            arg(Arg::new("ID").
                help("ID of the target item").
                value_parser(value_parser!(u32)).
                required(true).
                index(1))).
        
        //$ dolist notdone 1
        subcommand(Command::new("notdone").
            about("Set a Item's Status as NotDone").

            arg(Arg::new("ID").
                help("ID of the target item").
                value_parser(value_parser!(u32)).
                required(true).
                index(1))).
        
        //$ dolist delete 1
        subcommand(Command::new("delete").
            about("Delete a Item").

            arg(Arg::new("ID").
                help("ID of the target item").
                value_parser(value_parser!(u32)).
                required(true).
                index(1))).
        
        //$ dolist edit 1 -n "Chicken jerky" -d "buy some chicken jerky from movies"
        subcommand(Command::new("edit").
            about("Edit a Item").

            arg(Arg::new("ID").
                help("ID of the target item").
                value_parser(value_parser!(u32)).
                required(true).
                index(1)).
        
            arg(Arg::new("name").
                help("New name for the item").
                long("name").
                short('n')).

            arg(Arg::new("description").
                help("New description for the item").
                long("description").
                short('d'))).
        

        get_matches();


    match matches.subcommand() {
        Some(("new", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            let description = match sub_matches.get_one::<String>("description") {
                Some(desc) => desc,
                None => ""
            };
            create_new_item(name, description);
        }

        Some(("list", sub_matches)) => {
            let page = sub_matches.get_one::<u32>("page").unwrap();
            let limit = sub_matches.get_one::<u32>("limit").unwrap();
            let all  = sub_matches.get_flag("all");
            show_list_items(page,limit,all);
        }

        Some(("done", sub_matches)) => {
            let id = sub_matches.get_one::<u32>("ID").unwrap();
            set_item_status(id,&true);
        }

        Some(("notdone", sub_matches)) => {
            let id = sub_matches.get_one::<u32>("ID").unwrap();
            set_item_status(id,&false);
        }

        Some(("delete", sub_matches)) => {
            let id = sub_matches.get_one::<u32>("ID").unwrap();
            delete_item(id);
        }
        
        Some(("edit", sub_matches)) => {
            let id = sub_matches.get_one::<u32>("ID").unwrap();
            let name = match sub_matches.get_one::<String>("name") {
                Some(name) => name,
                None => ""
            };
            let description = match sub_matches.get_one::<String>("description") {
                Some(desc) => desc,
                None => ""
            };
            edit_item(id,name,description);
        }

        _ => {
            println!("No command specified.");
            println!("Run 'dolist --help' for usage information.");
        }
    }
}

fn create_new_item(name: &str, description: &str) {
    let app = get_db();

    app.add_item(name, description).expect("Couldn't add the item");
    println!("\n{}", "Added the new item to todo list".green());
}

fn show_list_items(page: &u32, limit: &u32, show_all: bool) {

    if page <= &0 {
        println!("The page number should be a positive number");
        return;
    }

    if limit <= &0 {
        println!("The limit number should be a positive number");
        return;
    }

    let app = get_db();

    let items = if show_all { app.show_all_items() } else { app.show_items(page,limit) };

    if items.is_err() {
        panic!("Can't get the items from database.");
    }

    let items =  items.unwrap();

    if items.is_empty() {
        if *page == 1 {
            println!("No todo items to show.");
            return;
        }
        println!("{}","Out of range page number.".red());
        return;
    }

    let total_items_in_table = app.get_total_number_of_items().unwrap_or(0);

    if show_all {
        println!("\n\tViewing All {} items\n",total_items_in_table.to_string().red())
    } else {
    let total_pages:f32 = (total_items_in_table as f32 / *limit as f32).ceil();
        println!("\n\tPage {} of {} With {} Total items ({} items per page)\n",page.to_string().bright_green(),total_pages.to_string().yellow(),total_items_in_table.to_string().red(),limit.to_string().purple());
    }

    let mut table = Table::new(items);

    table.with(Style::modern()).with(Colorization::exact([Color::FG_BLUE, Color::BOLD], Rows::first()));

    println!("{table}");
}

fn set_item_status(id: &u32,status: &bool) {
    let app = get_db();
    let effected_rows =  app.set_item_status(id,status).unwrap_or(0);

    if effected_rows == 0 {
        eprint!("{}","No Item with that ID found.".red());
        return;
    }

    println!("Item with id of {} got set as {}",id.to_string().bright_blue(),if *status {"Done".green()} else {"NotDone".red()});
}

fn delete_item(id: &u32) {
    let app = get_db();
    let effected_rows =  app.delete_item(id).unwrap_or(0);

    if effected_rows == 0 {
        eprint!("{}","No Item with that ID found.".red());
        return;
    }

    println!("Item With id of {} got {}",id.to_string().bright_blue(),"DELETED".red().bold());
}

fn edit_item(id: &u32,name: &str, description: &str) {
    let app = get_db();
    let effected_rows =  app.edit_item(id,name,description).unwrap_or(0);

    if effected_rows == 0 {
        eprint!("{}","No Item with that ID found.".red());
        return;
    }

    println!("Item With id of {} got {}",id.to_string().bright_blue(),"Edited".bright_purple());
}

fn get_db() -> Db {
    match Db::new() {
        Ok(db) => db,
        Err(_) => panic!("Can't connect to the database.")
    }
}