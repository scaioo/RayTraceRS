use crate::color;
use crate::color::Color;

#[derive(Clone, Debug)]
pub struct HDR{
    pub width : usize,
    pub height : usize,
    pub pixels : Vec<Color>,
}
