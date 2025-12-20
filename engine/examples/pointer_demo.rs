struct Data {
    pub value: i32,
}

fn main() {
    let mut data = Data { value: 10 };
    println!("Before: {}", data.value);
    let data_ptr = &mut data as *mut Data;
    unsafe {
        (*data_ptr).value += 5;
        println!("After: {}", (*data_ptr).value);
    }
    println!("Final: {}", data.value);
    let data_ref = &data;
    println!("Reference: {}", data_ref.value);
    let data_mut_ref = &mut data;
    data_mut_ref.value += 10;
    unsafe {
        (*data_ptr).value = 5;
    }
    println!("Mut Reference: {}", data_mut_ref.value);
}