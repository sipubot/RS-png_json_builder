extern crate image;
extern crate lodepng;
extern crate rgb;
extern crate image_base64;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate serde;

use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;

#[derive(Serialize, Deserialize)]
struct ThumbVec {
    pic : Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct Pic {
    data : String,
}

fn main() {
    //get argument from cgi
    let pathstr: Vec<String> = env::args().collect();
    let re_path = format!("{}{}", pathstr[1],"/result");
    //make result folder
    make_result_folder(&re_path);
    //getimage files
    let files = get_image_files(&pathstr[1]);
    //loading image
    let images = load_images(&files);
    //make thumbnail
    let thumbnails = resize_image(images.clone(), 30, 20);
    //make resize pic
    let resizeimages = resize_image(images.clone(), 450, 300);
    //save png
    save_pic(thumbnails, &re_path, "thumb_".to_string());
    save_pic(resizeimages, &re_path, "pic_".to_string());
    //save json
    json_parse(&re_path);
}

fn make_result_folder(path: &String) {
    fs::create_dir_all(&path).expect("makeFail fail");
}

fn get_image_files (pathstr: &String) -> Vec<fs::DirEntry>{
    let mut files = vec![];
    let path = Path::new(&pathstr);

    for entry in fs::read_dir(path).expect("Not found Directory") {
        let entry = entry.expect("unable get files");
        if entry.path().is_dir()  {
            continue;
        }
        if entry.path().extension().unwrap() == "png" {
            println!("{:?}",entry.path().display());
            files.push(entry);
        }
    };
    files
}

pub fn load_images (files: &Vec<fs::DirEntry>) -> Vec<image::DynamicImage> {
    let mut dis: Vec<image::DynamicImage> = Vec::new();
    for file in files.iter() {
        let di = image::open(&file.path()).unwrap();
        dis.push(di);
    }
    dis
}

pub fn resize_image (dy_imgs: Vec<image::DynamicImage>, width: u32, height: u32) -> Vec<image::DynamicImage>  {
    let mut r_imgs: Vec<image::DynamicImage> = Vec::new();
    for img in dy_imgs {
        let r_img = image::DynamicImage::resize(&img, width, height, image::Lanczos3);
        r_imgs.push(r_img);
    }
    r_imgs
}

pub fn save_pic (images: Vec<image::DynamicImage>, path: &String, name: String) {
    let mut i = 0;
    for img in images.iter() {
        let f = format!("{}/{}{}.{}",&path, &name, i.to_string(), "png");
        img.save(f).unwrap();
        i += 1;
    }
}

pub fn json_parse (_path: &String)  {
    let mut thumbv: Vec<String> = Vec::new();
    let path = Path::new(&_path);

    for entry in fs::read_dir(path.clone()).expect("Not found Directory") {
        let entry = entry.expect("unable get files");
        if entry.path().is_dir() {
            continue;
        }
        if entry.path().extension().unwrap() == "png" {
            let image_path = &entry.path().display().to_string();
            if image_path.clone().contains("thumb") {
                let base64 = image_base64::to_base64(&image_path.to_string()); 
                thumbv.push(base64[22..].to_string().replace("/",""));
            }
        }
    };
    //save Thumbnail
    let tv =  ThumbVec {
        pic: thumbv.clone(),
    };
    let json_t = serde_json::to_string(&tv).unwrap();
    savefilef(&format!("{}/{}.json",_path,"data"), &json_t);

    for entry in fs::read_dir(path.clone()).expect("Not found Directory") {
        let entry = entry.expect("unable get files");
        if entry.path().is_dir() || entry.path().extension().unwrap() != "png"{
            continue;
        }
        let image_path = &entry.path().display().to_string();
        if image_path.clone().contains("pic") {
            let base64 = image_base64::to_base64(&image_path.to_string()); 
            match &entry.path().file_name() {
                Some(file_name) => {
                    let idx = file_name.to_str().unwrap().chars().into_iter()
                        .filter(|a|a.is_digit(10))
                        .map(|a|a.to_string()).collect::<Vec<String>>()
                        .join("").parse::<usize>().unwrap();
                    let picfilename = &thumbv[idx][60..90];
                    let pic = Pic {
                        data : base64[22..].to_string(),
                    };
                    let json_p = serde_json::to_string(&pic).unwrap();
                    savefilef(&format!("{}/{}.json",_path,picfilename), &json_p);
                },
                _ => {
                    panic!("err");
                },
            };
        }
    
    };
}

fn savefilef (filename:&String, content:&String) {
    let path = Path::new(&filename);
    let display = path.display();
    let strings = &content;
    let mut file = match File::create(&path) {
        Err(e) => panic!("couldn't create {}:{}", display, e.description()),
        Ok(file) => file,
    };
    match file.write_all(strings.as_bytes()) {
        Err(e) => panic!("couldn't write to {}: {}", display, e.description()),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}
