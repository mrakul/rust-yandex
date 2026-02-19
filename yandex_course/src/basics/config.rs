pub const DEFAULT_COURSE_NAME: &str = "Rust для действующих разработчиков";

pub enum CourseCohort {
    Start,
    Base,
    Blockchain,
}

pub struct CourseConfig {
    pub cohort: CourseCohort,
}