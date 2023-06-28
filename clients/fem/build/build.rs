use std::{env, fs::{File, self}, path::Path, io::Read, fmt::Display};

use apache_arrow::{self as arrow,array::{StringArray, LargeStringArray}, record_batch::RecordBatchReader};
use bytes::Bytes;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use zip::ZipArchive;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("No suitable record in file")]
    NoRecord,
    #[error("No suitable data in file")]
    NoData,
    #[error("Cannot read arrow table")]
    ReadArrow(#[from] arrow::error::ArrowError),
    #[error("Cannot read parquet file")]
    ReadParquet(#[from] parquet::errors::ParquetError),
    #[error("Cannot find archive in zip file")]
    Zip(#[from] zip::result::ZipError),
    #[error("Cannot read zip file content")]
    ReadZip(#[from] std::io::Error),
}

mod names;
pub use names::{Name,Names};

pub struct GetIO<'a>{
    kind: String,
    variants: &'a Names,
}
impl<'a> GetIO<'a> {
    pub fn new<S: Into<String>>(kind: S, variants: &'a Names) -> Self {
        Self { kind: kind.into(), variants}
    }
}
impl<'a> Display for GetIO<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arms = self.variants.iter()
            .map(|name|
            format!(r#""{0}" => Ok(Box::new(SplitFem::<{1}>::new()))"#,
                name,name.variant()))
            .collect::<Vec<String>>().join(",\n");
        write!(f,"
        impl TryFrom<String> for Box<dyn Get{io}> {{
            type Error = FemError;
            fn try_from(value: String) -> std::result::Result<Self, Self::Error> {{
                match value.as_str() {{
                    {arms},
                    _ => Err(FemError::Convert(value)),
                }}
            }}
         }}
        ",io=self.kind,arms=arms)?;

        let variants = self.variants.iter()
        .map(|name|
        format!(r#"
        if let Some(x) = SplitFem::<{1}>::get_{0}(self) {{
            return x.serialize(s);
        }}
        "#,
            self.kind.to_lowercase(),name.variant()))
        .collect::<Vec<String>>().join("\n");
    write!(f,r##"
#[cfg(feature = "serde")]
impl serde::Serialize for Box<dyn Get{io}> {{
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {{
        {variants}
        Err(serde::ser::Error::custom(
            "failed to downcast `SplitFem<U>` with `U` as actors {io}puts",
        ))
    }}
}}
    "##,io=self.kind,variants=variants)?;

    let variants = self.variants.iter()
    .map(|name|
    format!(r#"
    "{0}" => Ok(Box::new(SplitFem::<{0}>::ranged(
        deser.range.clone(),
    )))
    "#,
        name.variant()))
    .collect::<Vec<String>>().join(",\n");
write!(f,r##"
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Box<dyn Get{io}> {{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {{
        let deser = super::SplitFemErased::deserialize(deserializer)?;
        match deser.kind.as_str() {{
            {variants},
            _ => Err(serde::de::Error::custom(
                "failed to deserialize into `SplitFem<U>` with `U` as actors {io}puts",
            ))
        }}
    }}
}}
    "##,io=self.kind,variants=variants)?;

        Ok(())
    }
} 
use std::sync::Arc;
use apache_arrow::datatypes::Schema;
use apache_arrow::record_batch::RecordBatch;
fn get_data(field: &str, fem_io: &str, schema: Arc<Schema>, table: &RecordBatch) -> Option<Vec<String>> {
    let (idx, _) = schema.column_with_name(field).expect(&format!(
        r#"failed to get {}puts "{}" index with field:\n{:}"#,
        fem_io,
        field,
        schema.field_with_name(field).unwrap()
    ));
                    match schema.field_with_name(field).unwrap().data_type() {
                    arrow::datatypes::DataType::Utf8 => table
                        .column(idx)
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .expect(&format!(
                            r#"failed to get {}puts "group" data at index #{} from field\n{:}"#,
                            fem_io,
                            idx,
                            schema.field_with_name("group").unwrap()
                        ))
                        .iter()
                        .map(|x| x.map(|x| x.to_owned()))
                        .collect(),
                    arrow::datatypes::DataType::LargeUtf8 => table
                        .column(idx)
                        .as_any()
                        .downcast_ref::<LargeStringArray>()
                        .expect(&format!(
                            r#"failed to get {}puts "group" data at index #{} from field\n{:}"#,
                            fem_io,
                            idx,
                            schema.field_with_name("group").unwrap()
                        ))
                        .iter()
                        .map(|x| x.map(|x| x.to_owned()))
                        .collect(),
                    other => panic!(
                        r#"Expected "Uft8" or "LargeUtf8" datatype, found {}"#,
                        other
                    ),
                }
}

// Read the fields
fn get_fem_io(zip_file: &mut ZipArchive<File>, fem_io: &str) -> Result<Names,Error> {
    println!("FEM_{}PUTS", fem_io.to_uppercase());
    let Ok(mut input_file) = zip_file.by_name(&format!(
        "rust/modal_state_space_model_2ndOrder_{}.parquet",
        fem_io
    )) else {
        panic!(r#"cannot find "rust/modal_state_space_model_2ndOrder_{}.parquet" in archive"#,fem_io)
    };
    let mut contents: Vec<u8> = Vec::new();
    input_file.read_to_end(&mut contents)?;

    let Ok(parquet_reader) = 
     ParquetRecordBatchReaderBuilder::try_new(Bytes::from(contents))
    else { panic!("failed to create `ParquetRecordBatchReaderBuilder`") };
    let Ok(parquet_reader) = 
        parquet_reader.with_batch_size(2048).build() 
    else { panic!("failed to create `ParquetRecordBatchReader`")};
    let schema = parquet_reader.schema();

    parquet_reader
    .map(|maybe_table| {
        if let Ok(table) = maybe_table {
            get_data("group", fem_io, schema.clone(), &table).zip(get_data("description", fem_io, schema.clone(), &table))
            .ok_or(Error::NoData)
        } else {
            Err(Error::NoRecord)
        }
    })
    .collect::<Result<Vec<_>, Error>>()
    .map(|data| {
        let (n,d) : (Vec<_>,Vec<_>) = data.into_iter().unzip();
        let n : Vec<_>= n.into_iter().flatten().collect();
        let d : Vec<_>= d.into_iter().flatten().collect();
        (n,d)

    })
    // .map(|data| data.into_iter().flatten().collect::<Vec<_>>())
    .map(|data| {
        let (name,description) = data;
        let mut data_iter = name.into_iter();
        let mut description_iter = description.into_iter();
        let mut name = data_iter.next().unwrap();
        let mut names: Vec<Name> = vec![Name::from(&name)];
        names.last_mut().map(|name| name.push_description(description_iter.next().unwrap()));
        loop {
            match data_iter.next() {
                Some(data) if data==name => {
                    names.last_mut().map(|name| name.push_description(description_iter.next().unwrap()));
                },
                Some(data) => {
                    name = data;
                    names.push(name.as_str().into());
                    names.last_mut().map(|name| name.push_description(description_iter.next().unwrap()));

                },
                None => break,
            }
        }
        names.into_iter().collect()
/*         data.dedup();
        data.into_iter()
            .enumerate()
            .map(|(k, fem_io)| {
                let name = Name(fem_io);
                println!(" #{:03}: {:>32} <=> {:<32}", k, name, name.variant());
                name
            })
            .collect() */
    })
}

fn main() -> anyhow::Result<()> {
    let (input_names,output_names) : (Names,Names) = 
    if let Ok(fem_repo) = env::var("FEM_REPO"){
    // Gets the FEM repository
    println!(
        "cargo:warning=generating FEM/Actors interface code based on the FEM inputs and outputs tables in {}",
        fem_repo
    );
    // Opens the mat file
    let path = Path::new(&fem_repo);
    let Ok(file) = File::open(path.join("modal_state_space_model_2ndOrder.zip")) 
    else {
        panic!("Cannot find `modal_state_space_model_2ndOrder.zip` in `FEM_REPO`");
    };
    let mut zip_file = zip::ZipArchive::new(file)?;

    let Ok(input_names) = get_fem_io(&mut zip_file, "in") 
    else {panic!("failed to parse FEM inputs variables")};
    let Ok(output_names) = get_fem_io(&mut zip_file, "out") 
    else {panic!("failed to parse FEM outputs variables")};
    (input_names,output_names)
} else {
    println!("cargo:warning=the FEM_REPO environment variable is not set, using dummy inputs and outputs instead");
    let (inputs,outputs): (Vec<_>,Vec<_>) = (1..=5)
    .map(|i| (String::from(format!("In{i}")),String::from(format!("Out{i}")))).unzip();
    (inputs.into_iter().collect(),outputs.into_iter().collect())
};

let out_dir = env::var_os("OUT_DIR").unwrap();
let dest_path = Path::new(&out_dir);

fs::write(dest_path.join("fem_actors_inputs.rs"), format!("{}", input_names))?;
fs::write(dest_path.join("fem_actors_outputs.rs"), format!("{}", output_names))?;

fs::write(dest_path.join("fem_get_in.rs"), format!("{}", GetIO::new("In",&input_names)))?;
fs::write(dest_path.join("fem_get_out.rs"), format!("{}", GetIO::new("Out",&output_names)))?;

fs::write(dest_path.join("fem_inputs.rs"), input_names.iter().map(|name| {
    format!(
        "{}",
        name.impl_enum_variant_for_io("Inputs")
    )
}).collect::<Vec<String>>().join("\n"))?;
fs::write(dest_path.join("fem_outputs.rs"), output_names.iter().map(|name| {
    format!(
        "{}",
        name.impl_enum_variant_for_io("Outputs")
    )
}).collect::<Vec<String>>().join("\n"))?;

if option_env!("FEM_REPO").is_some() {
    println!("cargo:rustc-cfg=fem");
}    
println!("cargo:rerun-if-changed=build");
println!("cargo:rerun-if-env-changed=FEM_REPO");
    Ok(())
}
