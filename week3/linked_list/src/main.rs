use linked_list::LinkedList;
pub mod linked_list;

fn main() {
    let mut list: LinkedList<&str> = LinkedList::new();
    assert!(list.is_empty());
    assert_eq!(list.get_size(), 0);
    for i in vec!["capture", "the", "flag"] {
        list.push_front(i);
    }
    let raw_list = list.clone();

    println!("{}", list);
    println!("list size: {}", list.get_size());
    println!("top element: {}", list.pop_front().unwrap());
    println!("{}", list);
    println!("size: {}", list.get_size());
    println!("{}", list.to_string()); // ToString impl for anything impl Display

    // If you implement iterator trait:
    println!("\nIterate the list:");
    for val in &raw_list {
        println!("{}", val);
    }
}
