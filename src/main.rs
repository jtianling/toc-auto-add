extern crate regex;
use regex::Regex;
use std::io::prelude::*;
use std::io;
use std::io::SeekFrom;
use std::env;
use std::iter::*;
use std::fmt;
use std::fs::OpenOptions;

struct Title {
  title: String,
  sub_titles: Vec<String>,
}

impl fmt::Display for Title {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut sub_title_string = String::new();

    sub_title_string.push_str("(");
    for s in &self.sub_titles {
      sub_title_string.push_str(&(s.clone()));
      sub_title_string.push_str(",");
    }

    sub_title_string.push_str(")");

    write!(f, "(Title: {}, subTitle: {}", self.title, sub_title_string)
  }
}

fn get_file_titles(file_content: &str) -> Vec<Title> {
  let mut file_titles: Vec<Title> = vec![];

  let mut line_index = 0;
  let first_level_title_re = Regex::new(r"^# (.+)$").unwrap();
  let second_level_title_re = Regex::new(r"^## (.+)$").unwrap();

  for line in file_content.lines() {
    //println!("{}: {}", line_index, line);
    
    if first_level_title_re.is_match(line) {
      println!("{}, Matched First level title, line: {}", line_index, line);
      let title_string: &str = first_level_title_re.captures(line).unwrap().at(1).unwrap();
      println!("{}, First level title: {}", line_index, title_string);
      let t = Title{ title: String::from(title_string), sub_titles: vec![] };
      file_titles.push(t);
      continue;
    }

    if second_level_title_re.is_match(line) {
      println!("{}, Second level title: {}", line_index, line);

      let title_string: &str = second_level_title_re.captures(line).unwrap().at(1).unwrap();
      match file_titles.last_mut() {
        Some(elem) => {
          println!("{}, Second level title pushed: {}", line_index, title_string);
          elem.sub_titles.push(String::from(title_string));
        }
        None => {
          panic!("Appear a second level title without a first level title");
        }
      }
    }

    line_index += 1;
  }

  return file_titles;
}

fn create_toc(file_titles: &Vec<Title>) -> String {
  let mut toc = String::new();

  toc = toc + "*目录*:\n";
  for title in file_titles {
    println!("title: {}", title);
    toc = toc + "*" + &title.title + "\n";

    for sub_title in &title.sub_titles {
      toc = toc + " *" + &(sub_title) + "\n";
    }
  }

  println!("title markdown string: {}", toc);
  return toc;
}

fn add_or_replace_toc(file_content: &str, toc: &str) -> String {
  let title_area_re = Regex::new(r"(?s)(?P<head><!-- toc-begin -->\n)(?P<body>.+)(?P<tail><!-- toc-end -->)").unwrap();

  // has toc already, so update the toc
  if title_area_re.is_match(file_content) {
    let title_area_string: &str = title_area_re.captures(file_content).unwrap().at(2).unwrap();
    println!("File exist toc area: {}", title_area_string);

    // ugly way to create replacer, because &String not really equal to &str
    let title_replacer: &str = &("$head".to_string() + toc + "$tail");

    let replaced_file_content = title_area_re.replace(file_content, title_replacer);
    println!("File content after toc replace: {}", replaced_file_content);
    return replaced_file_content;
  }

  return file_content.to_string();
}

fn process_file(filename: &str) -> Result<(), io::Error> {
  let mut file = try!(OpenOptions::new().read(true).write(true).open(filename));
  //let mut file = try!(File::open(filename.to_string()));
  let mut file_content = String::new();
  try!(file.read_to_string(&mut file_content));
  
  let file_titles: Vec<Title> = get_file_titles(&file_content);
  let toc = create_toc(&file_titles);
  let file_content_with_toc = add_or_replace_toc(&file_content, &toc);

  let bytes = file_content_with_toc.into_bytes();
  try!(file.seek(SeekFrom::Start(0)));
  try!(file.write_all(&bytes));
  try!(file.sync_all());

  Ok(())
}

fn get_filenames_from_cmd() -> Vec<String> {
  return env::args().skip(1).collect();
}

fn main() {
  let filenames = get_filenames_from_cmd();

  for filename in filenames {
    println!("Begin prcocess {}", filename);
    match process_file(&filename) {
      Ok(_) => {
        println!("Result: sucesss");
      }
      Err(e) => {
        println!("Error: {}", e);
      }
    }

    println!("End prcocess {}", filename);
  }

}
