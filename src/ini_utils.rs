// parse distro ini file of distrobox-assemble

use std::collections::HashMap;

pub fn from_ini(input: &str) -> Vec<(String, Vec<(String, String)>)> {
    #[inline]
    fn remove_quotes(s: &str) -> String {
        let s = s.trim();
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();

        if len >= 2 {
            let (first, last) = (chars[0], chars[len - 1]);
            let count = match (first, last) {
                ('\'', '\'') => chars.iter().filter(|&c| *c == '\'').count(),
                ('"', '"') => chars.iter().filter(|&c| *c == '"').count(),
                _ => return s.to_string(),
            };

            if count <= 2 {
                return s[1..len - 1].to_string();
            }
        }

        s.to_string()
    }
    let mut result: Vec<(String, Vec<(String, String)>)> = Vec::new();
    let mut current_section = String::new();

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_string();
            result.push((current_section.clone(), Vec::new()));
        } else if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();
            if current_section.is_empty() {
                panic!("Key-value pair outside of a section: {}", line);
            }
            if let Some((_, section)) = result.iter_mut().find(|(name, _)| name == &current_section)
            {
                section.push((remove_quotes(key), remove_quotes(value)));
            }
        }
    }

    result
}

pub fn to_ini(data: &[(String, Vec<(String, String)>)]) -> String {
    let mut output = String::new();

    for (section_name, section_data) in data.iter() {
        output.push_str(&format!("[{}]\n", section_name));

        for (key, value) in section_data.iter() {
            output.push_str(&format!("{}={}\n", key, value));
        }

        output.push('\n');
    }

    output
}

pub fn merge_ini(
    data: Vec<(String, Vec<(String, String)>)>,
) -> HashMap<String, HashMap<String, Vec<String>>> {
    let mut merged_data = HashMap::new();

    for (section_name, section_data) in data {
        let section_map = merged_data.entry(section_name).or_insert_with(HashMap::new);

        for (key, value) in section_data {
            let values = section_map.entry(key).or_insert_with(Vec::new);
            values.push(value);
        }
    }

    merged_data
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ini_content() -> String {
        "\
[riscv64-debian]
start_now=true
additional_packages=neofetch locales
additional_packages=git

[another-section]
key1=value1
key2=value2

[empty-section]
"
        .to_string()
    }

    fn sample_ini_data() -> Vec<(String, Vec<(String, String)>)> {
        let riscv64_debian = vec![
            ("start_now".to_string(), "true".to_string()),
            (
                "additional_packages".to_string(),
                "neofetch locales".to_string(),
            ),
            ("additional_packages".to_string(), "git".to_string()),
        ];

        let another_section = vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ];

        let empty_section = vec![];

        vec![
            ("riscv64-debian".to_string(), riscv64_debian),
            ("another-section".to_string(), another_section),
            ("empty-section".to_string(), empty_section),
        ]
    }

    #[test]
    fn test_from_ini() {
        let content = sample_ini_content();
        let parsed_data = from_ini(&content);
        let expected_data = sample_ini_data();

        assert_eq!(parsed_data, expected_data);
    }

    #[test]
    fn test_to_ini() {
        let data = sample_ini_data();
        let written_content = to_ini(&data);
        let expected_content = sample_ini_content();

        // Filter out empty lines from written_content and expected_content
        let filtered_written_lines: Vec<_> = written_content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        let filtered_expected_lines: Vec<_> = expected_content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();

        assert_eq!(
            filtered_written_lines, filtered_expected_lines,
            "The written content does not match the expected content."
        );
    }

    #[test]
    fn test_merge_ini_data() {
        let input = sample_ini_data();
        let output = merge_ini(input);

        let mut expected_output = HashMap::new();

        // riscv64-debian section
        let mut riscv64_debian = HashMap::new();
        riscv64_debian.insert("start_now".to_string(), vec!["true".to_string()]);
        riscv64_debian.insert(
            "additional_packages".to_string(),
            vec!["neofetch locales".to_string(), "git".to_string()],
        );
        expected_output.insert("riscv64-debian".to_string(), riscv64_debian);

        // another-section
        let mut another_section = HashMap::new();
        another_section.insert("key1".to_string(), vec!["value1".to_string()]);
        another_section.insert("key2".to_string(), vec!["value2".to_string()]);
        expected_output.insert("another-section".to_string(), another_section);

        // empty-section
        let empty_section = HashMap::new();
        expected_output.insert("empty-section".to_string(), empty_section);

        assert_eq!(
            output, expected_output,
            "The output does not match the expected output."
        );
    }
}
