extern crate image;
extern crate lodepng;
extern crate rgb;
extern crate base64;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate serde;

use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

struct PicObj {
    path : String,
    thumbstr : String,
    key : String,
}
#[derive(Serialize, Deserialize)]
struct Pic {
    data : String,
}
#[derive(Serialize, Deserialize)]
struct ThumbVec {
    pic : Vec<String>,
}

fn main() {
    //get argument from cgi
    let path_str: Vec<String> = env::args().collect();
    let result_path = format!("{}{}", path_str[1],"/result");
    //make result folder
    make_result_folder(&result_path);
    
    //parse thumb
    let obj: Vec<PicObj> = make_image_obj(&path_str[1], &result_path);
    //parse json
    make_json(&obj, &result_path);

}

fn make_result_folder(path: &String) {
    fs::create_dir_all(&path).expect("makeFail fail");
}

fn make_image_obj (path_str: &String, result_path: &String) -> Vec<PicObj> {
    let mut re: Vec<PicObj> = vec![];
    let mut idx = 0;

    let path = Path::new(&path_str);
    for entry in fs::read_dir(path).expect("Not found Directory") {
        let entry = entry.expect("unable get files");
        if entry.path().is_dir()  {
            continue;
        }
        if entry.path().extension().unwrap() == "png" {
            let _path_png = entry.path().display().to_string();
            let _thumb = load_images(&_path_png);
            let _key = save_thumb(&_thumb, &result_path, &idx);
            re.push(
                PicObj {
                    path : _path_png,
                    thumbstr : _key.1,
                    key : _key.0,
                }
            );
            idx += 1;
            println!("{:?}",entry.path().display());
        }
    };
    re
}

fn load_images (path: &String) -> image::DynamicImage {
    let di = image::open(&path).unwrap();
    image::DynamicImage::resize(&di, 30, 20, image::imageops::FilterType::Lanczos3)
}

fn save_thumb (img: &image::DynamicImage, path: &String, i: &i32) -> (String, String) {
    let f = format!("{}/thumb_{}.png",&path, i.to_string());
    img.save(f.clone()).unwrap();

    let buf = img.as_bytes().to_vec();
    let base64 = base64::encode(&buf)[22..].to_string(); 
    let key = &base64[60..90].replace("/","");
    return (key.to_string(), base64)
}

fn make_json (obj : &Vec<PicObj>, result_path: &String) {
    for o in obj.clone().into_iter() {
        let cpath = copy_to_rename(&o.path, &result_path, &o.key);

        let img = image::open(&cpath).unwrap();
        let buf = img.as_bytes().to_vec();
        let base64 = base64::encode(&buf)[22..].to_string(); 
        let pic = Pic {
            data : base64.to_string(),
        };
        let json_p = serde_json::to_string(&pic).unwrap();
        savefilef(&format!("{}/{}.json",&result_path,&o.key), &json_p);
    }
    //make data.json make
    let data = ThumbVec {
        pic : obj.into_iter().map(|a|a.thumbstr.clone()).collect()
    };
    let json_d = serde_json::to_string(&data).unwrap();
    savefilef(&format!("{}/data.json",&result_path), &json_d);
}

fn savefilef (filename:&String, content:&String) {
    let path = Path::new(&filename);
    let display = path.display();
    let strings = &content;
    let mut file = match File::create(&path) {
        Err(e) => panic!("couldn't create {}:{}", display, e),
        Ok(file) => file,
    };
    match file.write_all(strings.as_bytes()) {
        Err(e) => panic!("couldn't write to {}: {}", display, e),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

fn copy_to_rename(path: &String, re_path: &String, key: &String) -> String {
    let save_path = format!("{}/{}.png",re_path,key);
    let res = fs::copy(path, &save_path); 
    match res {
        Ok(v) => println!("copy to {} Ok!",v),
        Err(e) => println!("{:?}",e),
    };
    save_path
}