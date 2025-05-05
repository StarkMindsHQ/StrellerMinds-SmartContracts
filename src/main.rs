// Enum to represent the status of each module (Not Started, In Progress, Completed)
#[derive(Debug, Clone)]
enum ModuleStatus {
    NotStarted,
    InProgress,
    Completed,
}

// Struct to represent a Module
#[derive(Debug)]
struct Module {
    id: u32,
    name: String,
    status: ModuleStatus,
}

// Struct to represent a Course
#[derive(Debug)]
struct Course {
    id: u32,
    name: String,
    modules: Vec<Module>,
    completed: bool,
}

impl Course {
    // Method to calculate progress as a percentage
    fn calculate_progress(&self) -> f32 {
        let total_modules = self.modules.len() as f32;
        let completed_modules = self.modules.iter().filter(|m| matches!(m.status, ModuleStatus::Completed)).count() as f32;

        (completed_modules / total_modules) * 100.0
    }

    // Method to mark the course as completed if all modules are completed
    fn mark_course_completed(&mut self) {
        let all_completed = self.modules.iter().all(|m| matches!(m.status, ModuleStatus::Completed));

        if all_completed {
            self.completed = true;
            println!("Course '{}' is marked as completed.", self.name);
        } else {
            println!("Cannot mark '{}' as completed. Some modules are still incomplete.", self.name);
        }
    }
}

fn main() {
    // Create some sample modules
    let module1 = Module {
        id: 1,
        name: String::from("Module 1"),
        status: ModuleStatus::Completed,
    };
    let module2 = Module {
        id: 2,
        name: String::from("Module 2"),
        status: ModuleStatus::Completed,
    };
    let module3 = Module {
        id: 3,
        name: String::from("Module 3"),
        status: ModuleStatus::NotStarted,
    };

    // Create a sample course with modules
    let mut course = Course {
        id: 101,
        name: String::from("Rust Programming Basics"),
        modules: vec![module1, module2, module3],
        completed: false,
    };

    // Calculate and print progress
    let progress = course.calculate_progress();
    println!("Course Progress: {:.2}%", progress);

    // Attempt to mark the course as completed (it should fail)
    course.mark_course_completed();

    // Complete the last module
    course.modules[2].status = ModuleStatus::Completed;
    
    // Now attempt to mark the course as completed (it should succeed)
    course.mark_course_completed();
}
