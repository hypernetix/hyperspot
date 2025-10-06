//! Error catalog support (ErrDef for use with declare_errors! macro)

use crate::problem::Problem;

/// Static error definition from catalog
#[derive(Debug, Clone, Copy)]
pub struct ErrDef {
    pub status: u16,
    pub title: &'static str,
    pub code: &'static str,
    pub type_url: &'static str,
}

impl ErrDef {
    /// Convert this error definition into a Problem with the given detail
    #[inline]
    pub fn to_problem(&self, detail: impl Into<String>) -> Problem {
        Problem::new(self.status, self.title, detail.into())
            .with_code(self.code)
            .with_type(self.type_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn err_def_to_problem_works() {
        let def = ErrDef {
            status: 404,
            title: "Not Found",
            code: "TEST_NOT_FOUND",
            type_url: "https://errors.example.com/TEST_NOT_FOUND",
        };

        let problem = def.to_problem("Resource missing");
        assert_eq!(problem.status, 404);
        assert_eq!(problem.title, "Not Found");
        assert_eq!(problem.detail, "Resource missing");
        assert_eq!(problem.code, "TEST_NOT_FOUND");
        assert_eq!(
            problem.type_url,
            "https://errors.example.com/TEST_NOT_FOUND"
        );
    }
}
