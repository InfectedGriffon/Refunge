use crate::{stack::FungeStack, vector::FungeVector};

pub trait Stackable {
    fn pop(stack: &mut FungeStack<i32>) -> Self;
    fn push(stack: &mut FungeStack<i32>, val: Self);
}

impl Stackable for i32 {
    fn pop(stack: &mut FungeStack<i32>) -> Self {
        stack.pop()
    }

    fn push(stack: &mut FungeStack<i32>, val: Self) {
        stack.push(val)
    }
}
impl Stackable for char {
    fn pop(stack: &mut FungeStack<i32>) -> Self {
        char::from_u32(stack.pop() as u32).unwrap_or(' ')
    }

    fn push(stack: &mut FungeStack<i32>, val: Self) {
        stack.push(val as i32)
    }
}
impl Stackable for String {
    fn pop(stack: &mut FungeStack<i32>) -> Self {
        let mut output = String::new();
        loop {
            let c = stack.pop();
            if c == 0 {
                return output;
            }
            output.push(char::from_u32(c as u32).unwrap_or(' '));
        }
    }

    fn push(stack: &mut FungeStack<i32>, val: Self) {
        stack.push(0);
        val.chars().rev().for_each(|c| stack.push(c as i32));
    }
}
impl Stackable for FungeVector {
    fn pop(stack: &mut FungeStack<i32>) -> Self {
        let y = stack.pop();
        let x = stack.pop();
        FungeVector(x, y)
    }

    fn push(stack: &mut FungeStack<i32>, val: Self) {
        stack.push(val.0);
        stack.push(val.1);
    }
}
