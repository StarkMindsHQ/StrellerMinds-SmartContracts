use std::collections::HashMap;

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
            self.issue_certificate();
            self.emit_course_completion_event();
        } else {
            println!("Cannot mark '{}' as completed. Some modules are still incomplete.", self.name);
        }
    }

    // Issue a certificate upon course completion
    fn issue_certificate(&self) {
        println!("Certificate issued for course '{}'. Congratulations!", self.name);
    }

    // Emit course completion event
    fn emit_course_completion_event(&self) {
        println!("Event: Course '{}' completed successfully. Emitting course completion event.", self.name);
    }
}

// Global course registry (to simulate a collection of courses)
struct CourseRegistry {
    courses: HashMap<u32, Course>,
}

impl CourseRegistry {
    // Method to create a new course and add it to the registry
    fn create_course(&mut self, course: Course) {
        self.courses.insert(course.id, course);
    }

    // Method to mark course as completed by its ID
    fn mark_course_completed_by_id(&mut self, course_id: u32) {
        if let Some(course) = self.courses.get_mut(&course_id) {
            course.mark_course_completed();
        } else {
            println!("Course with ID {} not found.", course_id);
        }
    }
}

fn main() {
    // Create a course registry to manage multiple courses
    let mut course_registry = CourseRegistry {
        courses: HashMap::new(),
    };

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
    let course = Course {
        id: 101,
        name: String::from("Rust Programming Basics"),
        modules: vec![module1, module2, module3],
        completed: false,
    };

    // Add the course to the course registry
    course_registry.create_course(course);

    // Mark course as completed by its ID (this will fail initially)
    course_registry.mark_course_completed_by_id(101);

    // Complete the last module in the course
    if let Some(course) = course_registry.courses.get_mut(&101) {
        course.modules[2].status = ModuleStatus::Completed;
    }

    // Mark course as completed by its ID (this will succeed now)
    course_registry.mark_course_completed_by_id(101);
}
