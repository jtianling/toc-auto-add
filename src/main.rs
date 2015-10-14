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

  let first_level_title_re = Regex::new(r"^# (.+)$").unwrap();
  let second_level_title_re = Regex::new(r"^## (.+)$").unwrap();

  let mut is_in_code_area = false;
  let code_area_sign_re = Regex::new(r"^~~~").unwrap();

  for line in file_content.lines() {
    //println!("{}: {}", line_index, line);
    
    if code_area_sign_re.is_match(line) {
      println!("in or out code area: {}", line);
      is_in_code_area = !is_in_code_area;
    }

    if is_in_code_area {
      println!("Still in code area: {}", line);
      continue;
    }
    
    if first_level_title_re.is_match(line) {
      println!("Matched First level title, line: {}", line);
      let title_string: &str = first_level_title_re.captures(line).unwrap().at(1).unwrap();
      println!("First level title: {}", title_string);
      let t = Title{ title: String::from(title_string), sub_titles: vec![] };
      file_titles.push(t);
      continue;
    }

    if second_level_title_re.is_match(line) {
      println!("Second level title: {}", line);

      let title_string: &str = second_level_title_re.captures(line).unwrap().at(1).unwrap();
      match file_titles.last_mut() {
        Some(elem) => {
          println!("Second level title pushed: {}", title_string);
          elem.sub_titles.push(String::from(title_string));
        }
        None => {
          panic!("Appear a second level title without a first level title");
        }
      }
    }
  }

  return file_titles;
}

fn get_escaped_title_link(title: &str) -> String {
  println!("original title: {}", title);

  let trim_last_unuse_re = Regex::new(r"(\s|\.)$").unwrap();
  let trimed_title = trim_last_unuse_re.replace(&title, "");
  println!("trimed title: {}", trimed_title);

  let escape_re = Regex::new(r"\s|\.").unwrap();
  let escaped_title = escape_re.replace_all(&trimed_title, "-");
  println!("escaped title: {}", escaped_title);

  return escaped_title.to_lowercase();
}

fn create_toc(file_titles: &Vec<Title>) -> String {
  let mut toc = String::new();

  toc = toc + "**目录**:\n\n";
  for title in file_titles {
    println!("title: {}", title);
    toc = toc + "* [" + &title.title + "](#" + &get_escaped_title_link(&title.title) + ")\n";

    for sub_title in &title.sub_titles {
      toc = toc + " * [" + &(sub_title) + "](#" + &get_escaped_title_link(&sub_title) + ")\n";
    }
  }

  toc = toc + "\n";
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

  println!("File didn't exist toc area and try to add after more comment");

  let more_comment_re = Regex::new(r"<!-- more -->\n").unwrap();

  // didn't have toc but have more comment, so add toc after more comment
  if more_comment_re.is_match(file_content) {
    let more_comment_string: &str = more_comment_re.captures(file_content).unwrap().at(0).unwrap();
    println!("File exist more comment: {}", more_comment_string);

    // ugly way to create replacer, because &String not really equal to &str
    let title_replacer: &str = &("$0<!-- toc-begin -->\n".to_string() + toc + "<!-- toc-end -->\n");

    let replaced_file_content = more_comment_re.replace(file_content, title_replacer);
    println!("File content after toc replace: {}", replaced_file_content);
    return replaced_file_content;
  }

  return file_content.to_string();
}

fn process_file(filename: &str) -> Result<(), io::Error> {
  let mut file = try!(OpenOptions::new().read(true).write(true).open(filename));
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
