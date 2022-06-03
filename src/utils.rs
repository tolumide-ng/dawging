pub struct Utils;

impl Utils {
    pub fn split_to_vec(s: String) -> Vec<String> {
        s.split_terminator("").skip(1).collect::<Vec<_>>().iter().map(|x| x.to_string()).collect::<Vec<_>>()
    }
}