#[allow(warnings)]
mod bindings;

use bindings::exports::example::component::component_demo::Guest;
use bindings::exports::example::component::component_demo::{
    Address, Employee, Person, ResultCode,
};

struct Component;

impl Guest for Component {
    fn greet(name: String) -> String {
        format!("Hello, {}! Welcome to WebAssembly Components!", name)
    }

    fn create_person(name: String, age: u32) -> Person {
        let email = if age >= 18 {
            Some(format!("{}@example.com", name.to_lowercase().replace(" ", ".")))
        } else {
            None
        };

        Person { name, age, email }
    }

    fn format_person(p: Person) -> String {
        match p.email {
            Some(email) => format!("{} (age {}, email: {})", p.name, p.age, email),
            None => format!("{} (age {})", p.name, p.age),
        }
    }

    fn is_adult(p: Person) -> bool {
        p.age >= 18
    }

    fn get_people() -> Vec<Person> {
        vec![
            Person {
                name: "Alice".to_string(),
                age: 30,
                email: Some("alice@example.com".to_string()),
            },
            Person {
                name: "Bob".to_string(),
                age: 25,
                email: Some("bob@example.com".to_string()),
            },
            Person {
                name: "Charlie".to_string(),
                age: 15,
                email: None,
            },
        ]
    }

    fn find_person(name: String) -> Option<Person> {
        let people = Self::get_people();
        people.into_iter().find(|p| p.name == name)
    }

    fn validate_employee(emp: Employee) -> ResultCode {
        if emp.person.name.is_empty() {
            return ResultCode::InvalidInput("Name cannot be empty".to_string());
        }

        if emp.person.age < 16 {
            return ResultCode::InvalidInput("Employee must be at least 16 years old".to_string());
        }

        if emp.department.is_empty() {
            return ResultCode::InvalidInput("Department is required".to_string());
        }

        if emp.address.street.is_empty() || emp.address.city.is_empty() {
            return ResultCode::InvalidInput("Complete address is required".to_string());
        }

        ResultCode::Success
    }
}

bindings::export!(Component with_types_in bindings);
