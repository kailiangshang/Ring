pub struct SkillDef {
    pub name: &'static str,
    pub material_prompt: &'static str,
    pub summary_prompt: &'static str,
}

const SKILLS: &[SkillDef] = &[
    SkillDef {
        name: "decision",
        material_prompt: "You are assisting a decision-making session. Based on the session title and description, identify and collect relevant documents, data points, and graph nodes. For each material, create a concise summary. List pros, cons, risks, and options related to the decision topic.",
        summary_prompt: "Summarize this decision-making session. Include: 1) The key decision made, 2) Main arguments for and against, 3) Action items with owners, 4) Follow-up dates. Format as structured markdown.",
    },
    SkillDef {
        name: "research",
        material_prompt: "You are assisting a research session. Based on the session title and description, collect relevant resources, references, and existing knowledge from the graph. Identify gaps in knowledge and suggest areas to investigate.",
        summary_prompt: "Write a research report summarizing this session. Include: 1) Research question, 2) Key findings, 3) Data sources, 4) Conclusions, 5) Recommendations for further research. Format as structured markdown.",
    },
    SkillDef {
        name: "review",
        material_prompt: "You are assisting a review session. Based on the session title and description, collect the review targets (documents, code, designs). Identify review criteria and checklists relevant to the review type.",
        summary_prompt: "Summarize this review session. Include: 1) Items reviewed, 2) Key findings (issues and positive aspects), 3) Improvement suggestions with priority levels, 4) Agreed actions. Format as structured markdown.",
    },
    SkillDef {
        name: "retrospective",
        material_prompt: "You are assisting a retrospective session. Based on the session title and description, collect project timeline data, metrics, and previous retrospective outcomes from the graph. Identify key events and milestones.",
        summary_prompt: "Summarize this retrospective. Include: 1) What went well, 2) What could be improved, 3) Lessons learned, 4) Action items for next cycle. Format as structured markdown.",
    },
    SkillDef {
        name: "knowledge_sharing",
        material_prompt: "You are assisting a knowledge sharing session. Based on the session title and description, collect relevant materials, prior discussions, and graph nodes related to the topic. Organize materials into a logical flow for presentation.",
        summary_prompt: "Create organized notes from this knowledge sharing session. Include: 1) Key topics covered, 2) Important takeaways, 3) References and resources mentioned, 4) Open questions. Format as structured markdown.",
    },
];

pub fn get_skill(name: &str) -> Option<&'static SkillDef> {
    SKILLS.iter().find(|s| s.name == name)
}

pub fn build_material_system_prompt(
    skill_name: &str,
    session_title: &str,
    session_description: &str,
) -> Option<String> {
    let skill = get_skill(skill_name)?;
    Some(format!(
        "{}\n\nSession: {}\nDescription: {}\n\nAnalyze the topic and provide a structured list of materials that should be prepared for this session. For each material, specify: title, type (document/graph_node/ai_generated), and a brief description of what it should contain.",
        skill.material_prompt,
        session_title,
        if session_description.is_empty() {
            "N/A"
        } else {
            session_description
        },
    ))
}

pub fn build_summary_system_prompt(skill_name: &str) -> Option<String> {
    let skill = get_skill(skill_name)?;
    Some(skill.summary_prompt.to_string())
}
