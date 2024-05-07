mod metric;

use clap::Parser;
use indexmap::IndexMap;
use metric::Metric;

/// Git extension in order to attach metrics to commits
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the metric
    #[arg(short, long)]
    name: String,

    /// Information about the context
    #[arg(short, long)]
    context: Vec<String>,

    /// Value of the metrics
    #[arg(short, long)]
    value: f64,
}

impl Args {
    pub fn into_metric(self) -> Metric {
        let context = self
            .context
            .into_iter()
            .filter_map(|value| {
                value
                    .split_once(':')
                    .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
            })
            .collect::<IndexMap<String, String>>();

        Metric {
            name: self.name,
            context,
            value: self.value,
        }
    }
}

fn old_main() {
    let args = Args::parse();
    let metric = args.into_metric();
    println!("ARGS: {metric:?}");
}

fn main() {
    let repo = git2::Repository::open_from_env().expect("unable to read git repository");
    let head = repo.revparse("HEAD").expect("couldn't get HEAD");
    let notes = repo
        .notes(Some("refs/notes/metrics"))
        .expect("couldn't get metrics");
    for note in notes {
        println!("{note:?}");
    }
}
