//! The built-in template registry.
//!
//! Four original undergraduate project templates: a short course report, a
//! multi-file senior thesis skeleton, a problem set, and a lab notebook
//! entry. Every file body is original text written for this crate; none of
//! it is copied from a real university's template or from a third-party
//! document class. Each template uses only the standard LaTeX `article`
//! class and declares its packages explicitly in [`Template::packages`] —
//! never inside the body text, see [`crate::instantiate`].
//!
//! Every body below is a plain (non-format) string literal: the
//! `{{project_name}}`, `{{author}}`, and `{{packages}}` tokens are written
//! out literally and substituted later by [`crate::instantiate::instantiate`]
//! — they are not `format!` placeholders and need no brace-doubling.

use crate::manifest::{Template, TemplateFile};

/// Returns every built-in template, in a stable order.
pub fn all_templates() -> Vec<Template> {
    vec![
        course_report(),
        senior_thesis(),
        problem_set(),
        lab_notebook(),
    ]
}

/// Looks up a built-in template by [`Template::id`].
pub fn find_template(id: &str) -> Option<Template> {
    all_templates().into_iter().find(|t| t.id == id)
}

fn course_report() -> Template {
    Template {
        id: "course-report".into(),
        title: "Course Report".into(),
        description: "A short single-file report for a course assignment.".into(),
        packages: vec!["amsmath".into(), "graphicx".into(), "hyperref".into()],
        files: vec![TemplateFile::new(
            "main.tex",
            "\\documentclass[11pt]{article}\n\
             {{packages}}\n\
             \n\
             \\title{{{project_name}}}\n\
             \\author{{{author}}}\n\
             \\date{\\today}\n\
             \n\
             \\begin{document}\n\
             \\maketitle\n\
             \n\
             \\section{Overview}\n\
             State the question this report answers and summarize the result in two\n\
             or three sentences.\n\
             \n\
             \\section{Method}\n\
             Describe what you did: the setup, the data, or the derivation steps.\n\
             \n\
             \\section{Results}\n\
             Present the findings. Use a figure with \\verb|\\includegraphics| or a\n\
             table where a picture or numbers make the point faster than prose.\n\
             \n\
             \\section{Discussion}\n\
             Interpret the results and note any limitations.\n\
             \n\
             \\end{document}\n",
        )],
    }
}

fn senior_thesis() -> Template {
    Template {
        id: "senior-thesis".into(),
        title: "Senior Thesis".into(),
        description: "A multi-file undergraduate thesis skeleton with a titlepage and chapters."
            .into(),
        packages: vec![
            "amsmath".into(),
            "amssymb".into(),
            "graphicx".into(),
            "hyperref".into(),
            "geometry".into(),
            "titlesec".into(),
        ],
        files: vec![
            TemplateFile::new(
                "main.tex",
                "\\documentclass[12pt]{article}\n\
                 {{packages}}\n\
                 \\geometry{margin=1.25in}\n\
                 \n\
                 \\title{{{project_name}}}\n\
                 \\author{{{author}}}\n\
                 \\date{\\today}\n\
                 \n\
                 \\begin{document}\n\
                 \\maketitle\n\
                 \\tableofcontents\n\
                 \n\
                 \\input{chapters/introduction}\n\
                 \\input{chapters/conclusion}\n\
                 \n\
                 \\end{document}\n",
            ),
            TemplateFile::new(
                "chapters/introduction.tex",
                "\\section{Introduction}\n\
                 \n\
                 State the problem this thesis addresses, why it matters, and how the\n\
                 remaining sections are organized.\n",
            ),
            TemplateFile::new(
                "chapters/conclusion.tex",
                "\\section{Conclusion}\n\
                 \n\
                 Summarize what was shown and what a natural next step would be.\n",
            ),
            TemplateFile::new(
                "references.bib",
                "% Add your BibTeX entries here, e.g.:\n\
                 % @article{key1234,\n\
                 %   author  = {Last, First},\n\
                 %   title   = {Title},\n\
                 %   journal = {Journal},\n\
                 %   year    = {2026}\n\
                 % }\n",
            ),
        ],
    }
}

fn problem_set() -> Template {
    Template {
        id: "problem-set".into(),
        title: "Problem Set".into(),
        description: "A numbered problem set with a solution environment per problem.".into(),
        packages: vec!["amsmath".into(), "amssymb".into(), "enumitem".into()],
        files: vec![TemplateFile::new(
            "main.tex",
            "\\documentclass[11pt]{article}\n\
             {{packages}}\n\
             \n\
             \\newenvironment{solution}{\\par\\noindent\\textbf{Solution.}}{\\par}\n\
             \n\
             \\title{{{project_name}}}\n\
             \\author{{{author}}}\n\
             \\date{\\today}\n\
             \n\
             \\begin{document}\n\
             \\maketitle\n\
             \n\
             \\begin{enumerate}[label=\\textbf{Problem \\arabic*.}, wide]\n\
             \\item State the first problem here.\n\
             \\begin{solution}\n\
             Write the solution here.\n\
             \\end{solution}\n\
             \n\
             \\item State the second problem here.\n\
             \\begin{solution}\n\
             Write the solution here.\n\
             \\end{solution}\n\
             \\end{enumerate}\n\
             \n\
             \\end{document}\n",
        )],
    }
}

fn lab_notebook() -> Template {
    Template {
        id: "lab-notebook".into(),
        title: "Lab Notebook Entry".into(),
        description: "A dated entry for a single experimental session.".into(),
        packages: vec!["amsmath".into(), "graphicx".into(), "siunitx".into()],
        files: vec![TemplateFile::new(
            "main.tex",
            "\\documentclass[11pt]{article}\n\
             {{packages}}\n\
             \n\
             \\title{{{project_name}}}\n\
             \\author{{{author}}}\n\
             \\date{\\today}\n\
             \n\
             \\begin{document}\n\
             \\maketitle\n\
             \n\
             \\section*{Objective}\n\
             What this session set out to measure or test.\n\
             \n\
             \\section*{Setup}\n\
             Apparatus, materials, and configuration.\n\
             \n\
             \\section*{Procedure}\n\
             The steps actually followed, in order.\n\
             \n\
             \\section*{Data}\n\
             Raw measurements, e.g. \\(x = \\SI{1.23}{\\meter}\\).\n\
             \n\
             \\section*{Notes}\n\
             Anomalies, follow-ups, or ideas for next time.\n\
             \n\
             \\end{document}\n",
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_builtin_template_validates() {
        for t in all_templates() {
            t.validate()
                .unwrap_or_else(|e| panic!("template {:?} failed validation: {e}", t.id));
        }
    }

    #[test]
    fn ids_are_unique() {
        let templates = all_templates();
        let mut ids: Vec<&str> = templates.iter().map(|t| t.id.as_str()).collect();
        let len_before = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), len_before, "duplicate template id");
    }

    #[test]
    fn find_template_looks_up_by_id() {
        assert!(find_template("course-report").is_some());
        assert!(find_template("senior-thesis").is_some());
        assert!(find_template("problem-set").is_some());
        assert!(find_template("lab-notebook").is_some());
        assert!(find_template("does-not-exist").is_none());
    }

    fn temp_subdir(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "flashtex-project-templates-registry-test-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    /// Every `\usepackage` line that ends up in a generated file must come
    /// from that template's declared `packages`, and every declared package
    /// must actually be emitted — the two lists must match exactly. This is
    /// the acceptance criterion "required packages must be declared
    /// explicitly in the manifest, not implied by the body text",
    /// mechanically checked rather than eyeballed.
    #[test]
    fn declared_packages_exactly_match_generated_usepackage_lines() {
        for t in all_templates() {
            let dir = temp_subdir(&t.id);
            let options = crate::instantiate::InstantiateOptions {
                project_name: "Test Project".into(),
                author: "Test Author".into(),
                overwrite: false,
            };
            crate::instantiate::instantiate(&t, &dir, &options).unwrap();

            let mut found_packages: Vec<String> = Vec::new();
            for file in &t.files {
                let path = dir.join(&file.path);
                let contents = std::fs::read_to_string(&path).unwrap();
                for line in contents.lines() {
                    if let Some(rest) = line.trim().strip_prefix("\\usepackage{")
                        && let Some(name) = rest.strip_suffix('}')
                    {
                        found_packages.push(name.to_string());
                    }
                }
            }
            let mut declared = t.packages.clone();
            declared.sort_unstable();
            found_packages.sort_unstable();
            assert_eq!(
                declared, found_packages,
                "template {:?}: declared packages must exactly match generated \\usepackage lines",
                t.id
            );
            std::fs::remove_dir_all(&dir).ok();
        }
    }

    #[test]
    fn every_builtin_template_instantiates_with_a_unicode_project_name() {
        for t in all_templates() {
            let dir = temp_subdir(&format!("{}-unicode", t.id));
            let options = crate::instantiate::InstantiateOptions {
                project_name: "Métodos Numéricos — 数値解析 レポート".into(),
                author: "Étienne Müller".into(),
                overwrite: false,
            };
            let report = crate::instantiate::instantiate(&t, &dir, &options).unwrap();
            assert_eq!(report.written_files.len(), t.files.len());
            for path in &report.written_files {
                assert!(path.is_file());
            }
            std::fs::remove_dir_all(&dir).ok();
        }
    }
}
