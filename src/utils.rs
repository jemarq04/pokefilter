pub mod cli;
pub mod enums;
pub mod helpers;

#[macro_export]
macro_rules! impl_Display {
  ( $T:ty ) => {
    impl std::fmt::Display for $T {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self
          .to_possible_value()
          .expect("no values are skipped")
          .get_name()
          .fmt(f)
      }
    }
  };
}

#[macro_export]
macro_rules! svec {
  () => {vec![]};
  ( $elem:expr; $n:expr ) => {vec![$elem.to_string(); $n]};
  ( $($x:expr),+ $(,)? ) => {vec![$($x.to_string()),*]};
}

#[macro_export]
macro_rules! get_name {
  ( follow $T:expr, $client:ident, $lang:expr ) => {{
    let mut result = $T.name.clone();
    if let Ok(resource) = $T.follow(&$client).await {
      for name in resource.names.iter() {
        if let Ok(item) = name.language.follow(&$client).await
          && item.name == $lang
        {
          result = name.name.clone();
        }
      }
    }
    result
  }};
  ( $T:expr, $client:ident, $lang:expr ) => {{
    let mut result = $T.name.clone();
    for name in $T.names.iter() {
      if let Ok(item) = name.language.follow(&$client).await
        && item.name == $lang
      {
        result = name.name.clone();
      }
    }
    result
  }};
}

#[cfg(test)]
mod tests {
  use rustemon::client::RustemonClient;
  use std::fs::File;
  use std::io::{self, BufRead, BufReader, Lines, Read, Write};
  use std::path::Path;

  fn read_lines<P>(filename: P) -> io::Result<Lines<BufReader<File>>>
  where
    P: AsRef<Path>,
  {
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
  }

  fn kebab_to_pascal(name: &str) -> String {
    let mut result = String::new();
    let mut first_char_in_word = true;
    for c in name.chars() {
      if c == '-' {
        first_char_in_word = true;
        continue;
      } else if first_char_in_word {
        result.push(c.to_uppercase().next().unwrap());
        first_char_in_word = false;
      } else {
        result.push(c);
      }
    }

    result
  }

  fn is_same_file(file1: &Path, file2: &Path) -> Result<bool, std::io::Error> {
    let f1 = File::open(file1)?;
    let f2 = File::open(file2)?;

    // Check if file sizes are different
    if f1.metadata().unwrap().len() != f2.metadata().unwrap().len() {
      return Ok(false);
    }

    // Use buf readers since they are much faster
    let f1 = BufReader::new(f1);
    let f2 = BufReader::new(f2);

    // Do a byte to byte comparison of the two files
    for (b1, b2) in f1.bytes().zip(f2.bytes()) {
      if b1.unwrap() != b2.unwrap() {
        return Ok(false);
      }
    }

    Ok(true)
  }

  #[tokio::test]
  async fn check_value_enums() {
    let client = RustemonClient::default();

    // Create temporary file for comparison
    let outpath = Path::new("src/utils/enums_temp.rs");
    let mut outfile = match File::create(&outpath) {
      Ok(f) => f,
      Err(err) => panic!("could not open file {}: {}", outpath.display(), err),
    };
    let mut write_line = |x: &str| {
      if let Err(err) = outfile.write_all(format!("{}\n", x).as_bytes()) {
        panic!("could not write to {}: {}", outpath.display(), err);
      }
    };

    // Copy initial lines from enums file to temporary file
    let mut last_line = String::new();
    let inpath = Path::new("src/utils/enums.rs");
    match read_lines(inpath) {
      Err(err) => panic!("could not open file {}: {}", inpath.display(), err),
      Ok(lines) => {
        for line in lines.map_while(Result::ok) {
          if line.contains("pub enum") {
            break;
          }
          write_line(&line);
          last_line = line.clone();
        }
      },
    };

    // Generate ValueEnums from respective endpoints
    let derive_line = last_line;

    // -> LanguageId
    write_line("pub enum LanguageId {");
    let all_resources = rustemon::utility::language::get_all_entries(&client)
      .await
      .unwrap();
    for resource in all_resources.iter() {
      let name = resource.name.clone();
      let mut alias = String::new();
      for c in name.chars() {
        if c.is_ascii_uppercase() {
          alias = format!("  #[value(alias = \"{}\")]", name);
          break;
        }
      }
      if !alias.is_empty() {
        write_line(&alias);
      }
      write_line(&format!("  {},", kebab_to_pascal(&resource.name)));
    }
    write_line("}\nimpl_Display!(LanguageId);");

    match is_same_file(inpath, outpath) {
      Err(err) => panic!(
        "could not compare files {} and {}: {}",
        inpath.display(),
        outpath.display(),
        err
      ),
      Ok(false) => panic!("enums file is not up-to-date"),
      Ok(true) => {
        if let Err(err) = std::fs::remove_file(outpath) {
          panic!(
            "could not remove temporary file {}: {}",
            outpath.display(),
            err
          );
        }
      },
    };
  }
}
