// Copyright (c) 2016-2017 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::result;
use std::str::FromStr;
use std::fmt;

use message::{Persistable, Routable};
use message::originsrv::OriginPackage;
use originsrv::Pageable;
use protobuf::RepeatedField;
use regex::Regex;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use sharding::InstaId;

use error::ProtocolError;
pub use message::jobsrv::*;

pub const GITHUB_PUSH_NOTIFY_ID: u64 = 23;

impl Into<Job> for JobSpec {
    fn into(mut self) -> Job {
        let mut job = Job::new();
        job.set_owner_id(self.get_owner_id());
        job.set_state(JobState::default());
        job.set_project(self.take_project());
        if self.has_channel() {
            job.set_channel(self.take_channel());
        }
        job
    }
}

impl Routable for JobSpec {
    type H = InstaId;

    fn route_key(&self) -> Option<Self::H> {
        Some(InstaId(self.get_owner_id()))
    }
}

impl Routable for JobLogGet {
    type H = InstaId;

    fn route_key(&self) -> Option<Self::H> {
        Some(InstaId(self.get_id()))
    }
}

impl Routable for JobGet {
    type H = InstaId;

    fn route_key(&self) -> Option<Self::H> {
        Some(InstaId(self.get_id()))
    }
}

impl Routable for Job {
    type H = InstaId;

    fn route_key(&self) -> Option<Self::H> {
        Some(InstaId(self.get_id()))
    }
}

// Note: Given that we only run a single JobServer, the specific
// routing key for this message isn't really important (everything is
// going to route to the same, single place anyway). If we ever do run
// multiple JobServers, though, this may need to be revisited (as will
// other corners of the code).
impl Routable for ProjectJobsGet {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_name().to_string())
    }
}

impl Pageable for ProjectJobsGet {
    fn get_range(&self) -> [u64; 2] {
        [self.get_start(), self.get_stop()]
    }
}

impl Serialize for Job {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut strukt = serializer.serialize_struct("job", 10)?;

        // Technically, an ID is a 64-bit integer, but that can cause
        // issues when processing it in JavaScript on the front-end,
        // so we'll render it as a string instead.
        //
        // Ideally, we'd like to use some kind of declarative
        // approach, using a `#[serde(with = "...")]`
        // annotation. Since the Job struct is code-generated by the
        // protobuf machinery, though, we'd need to do something like
        // declare another struct that mirrors the structure of the
        // JSON output, add annotations to *that*, and then define a
        // conversion from the protobuf message struct into the JSON
        // struct. Maybe we can take that approach in a later PR and
        // treat all our structs consistently.
        strukt.serialize_field("id", &self.get_id().to_string())?;

        strukt.serialize_field("created_at", &self.get_created_at())?;

        // Technically, we could get the origin and name from the
        // package identifier, but we'll only have that if the job was
        // complete. The project information will always be present,
        // however.
        strukt.serialize_field(
            "origin",
            &self.get_project().get_origin_name(),
        )?;
        strukt.serialize_field(
            "name",
            &self.get_project().get_package_name(),
        )?;

        if self.has_package_ident() {
            let ident = self.get_package_ident();
            strukt.serialize_field("version", ident.get_version())?;
            strukt.serialize_field("release", ident.get_release())?;
        }

        if self.has_build_started_at() {
            strukt.serialize_field(
                "build_started_at",
                &self.get_build_started_at(),
            )?;
        }
        if self.has_build_finished_at() {
            strukt.serialize_field(
                "build_finished_at",
                &self.get_build_finished_at(),
            )?;
        }

        strukt.serialize_field("state", &self.get_state())?;

        if self.has_error() {
            strukt.serialize_field("error", self.get_error())?;
        }

        if self.has_channel() {
            strukt.serialize_field("channel", self.get_channel())?;
        }

        strukt.end()
    }
}

impl Serialize for ProjectJobsGetResponse {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut strukt = serializer.serialize_struct("project_jobs_get_response", 1)?;
        strukt.serialize_field("jobs", self.get_jobs())?;
        strukt.end()
    }
}

impl JobLog {
    /// Strip any ANSI control codes from the contents of the log
    /// chunk. Useful mainly for removing color codes.
    pub fn strip_ansi(&mut self) {
        lazy_static! {
            // https://github.com/chalk/ansi-regex/blob/master/index.js
            static ref RE: Regex = Regex::new(
                r"[\x1b\x9b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]")
                .unwrap();
        }

        let mut stripped = RepeatedField::new();
        for line in self.get_content() {
            let after = RE.replace_all(line, "");
            stripped.push(after);
        }

        self.set_content(stripped);
    }
}

impl Serialize for JobLog {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut log = serializer.serialize_struct("JobLog", 4)?;
        log.serialize_field("start", &self.get_start())?;
        log.serialize_field("stop", &self.get_stop())?;
        log.serialize_field("content", &self.get_content())?;
        log.serialize_field("is_complete", &self.get_is_complete())?;
        log.end()
    }
}

impl Default for JobState {
    fn default() -> JobState {
        JobState::Pending
    }
}

impl Serialize for JobState {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self as u64 {
            0 => serializer.serialize_str("Pending"),
            1 => serializer.serialize_str("Processing"),
            2 => serializer.serialize_str("Complete"),
            3 => serializer.serialize_str("Rejected"),
            4 => serializer.serialize_str("Failed"),
            5 => serializer.serialize_str("Dispatched"),
            6 => serializer.serialize_str("CancelPending"),
            7 => serializer.serialize_str("CancelProcessing"),
            8 => serializer.serialize_str("CancelComplete"),
            _ => panic!("Unexpected enum value"),
        }
    }
}

impl FromStr for JobState {
    type Err = ProtocolError;

    fn from_str(value: &str) -> result::Result<Self, Self::Err> {
        match value.to_lowercase().as_ref() {
            "pending" => Ok(JobState::Pending),
            "processing" => Ok(JobState::Processing),
            "complete" => Ok(JobState::Complete),
            "rejected" => Ok(JobState::Rejected),
            "failed" => Ok(JobState::Failed),
            "dispatched" => Ok(JobState::Dispatched),
            "cancelpending" => Ok(JobState::CancelPending),
            "cancelprocessing" => Ok(JobState::CancelProcessing),
            "cancelcomplete" => Ok(JobState::CancelComplete),
            _ => Err(ProtocolError::BadJobState(value.to_string())),
        }
    }
}

impl fmt::Display for JobState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match *self {
            JobState::Dispatched => "Dispatched",
            JobState::Pending => "Pending",
            JobState::Processing => "Processing",
            JobState::Complete => "Complete",
            JobState::Rejected => "Rejected",
            JobState::Failed => "Failed",
            JobState::CancelPending => "CancelPending",
            JobState::CancelProcessing => "CancelProcessing",
            JobState::CancelComplete => "CancelComplete",
        };
        write!(f, "{}", value)
    }
}

impl Persistable for Job {
    type Key = u64;

    fn primary_key(&self) -> Self::Key {
        self.get_id()
    }

    fn set_primary_key(&mut self, value: Self::Key) {
        self.set_id(value);
    }
}

impl Routable for JobGroupSpec {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(format!("{}/{}", self.get_origin(), self.get_package()))
    }
}

impl From<OriginPackage> for JobGraphPackage {
    fn from(value: OriginPackage) -> JobGraphPackage {
        let mut package = JobGraphPackage::new();

        let name = format!("{}", value.get_ident());
        let target = value.get_target().to_string();

        let deps = value.get_deps().iter().map(|x| format!("{}", x)).collect();

        package.set_ident(name);
        package.set_target(target);
        package.set_deps(deps);
        package
    }
}

impl From<JobGraphPackage> for JobGraphPackageCreate {
    fn from(value: JobGraphPackage) -> JobGraphPackageCreate {
        let mut package = JobGraphPackageCreate::new();

        let name = format!("{}", value.get_ident());
        let target = value.get_target().to_string();

        let deps = value.get_deps().iter().map(|x| format!("{}", x)).collect();

        package.set_ident(name);
        package.set_target(target);
        package.set_deps(deps);
        package
    }
}

impl Into<JobGraphPackage> for JobGraphPackagePreCreate {
    fn into(self) -> JobGraphPackage {
        let mut package = JobGraphPackage::new();

        let name = format!("{}", self.get_ident());
        let target = self.get_target().to_string();

        let deps = self.get_deps().iter().map(|x| format!("{}", x)).collect();

        package.set_ident(name);
        package.set_target(target);
        package.set_deps(deps);
        package
    }
}

impl Routable for JobGroupGet {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_group_id().to_string())
    }
}

impl Routable for JobGroupAbort {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_group_id().to_string())
    }
}
impl Routable for JobGroupCancel {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_group_id().to_string())
    }
}

impl Routable for JobGraphPackageCreate {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_ident().to_string())
    }
}

impl Routable for JobGraphPackagePreCreate {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_ident().to_string())
    }
}

impl Routable for JobGroupOriginGet {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_origin().to_string())
    }
}

impl Routable for JobGraphPackageStatsGet {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_origin().to_string())
    }
}

impl Routable for JobGraphPackageReverseDependenciesGet {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(format!("{}/{}", self.get_origin(), self.get_name()))
    }
}

impl fmt::Display for JobGroupState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match *self {
            JobGroupState::GroupDispatching => "Dispatching",
            JobGroupState::GroupPending => "Pending",
            JobGroupState::GroupComplete => "Complete",
            JobGroupState::GroupFailed => "Failed",
            JobGroupState::GroupQueued => "Queued",
            JobGroupState::GroupCanceled => "Canceled",
        };
        write!(f, "{}", value)
    }
}

impl FromStr for JobGroupState {
    type Err = ProtocolError;

    fn from_str(value: &str) -> result::Result<Self, Self::Err> {
        match value.to_lowercase().as_ref() {
            "dispatching" => Ok(JobGroupState::GroupDispatching),
            "pending" => Ok(JobGroupState::GroupPending),
            "complete" => Ok(JobGroupState::GroupComplete),
            "failed" => Ok(JobGroupState::GroupFailed),
            "queued" => Ok(JobGroupState::GroupQueued),
            "canceled" => Ok(JobGroupState::GroupCanceled),
            _ => Err(ProtocolError::BadJobGroupState(value.to_string())),
        }
    }
}

impl Serialize for JobGroupState {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self as u64 {
            0 => serializer.serialize_str("Pending"),
            1 => serializer.serialize_str("Dispatching"),
            2 => serializer.serialize_str("Complete"),
            3 => serializer.serialize_str("Failed"),
            4 => serializer.serialize_str("Queued"),
            5 => serializer.serialize_str("Canceled"),
            _ => panic!("Unexpected enum value"),
        }
    }
}

impl fmt::Display for JobGroupProjectState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match *self {
            JobGroupProjectState::NotStarted => "NotStarted",
            JobGroupProjectState::InProgress => "InProgress",
            JobGroupProjectState::Success => "Success",
            JobGroupProjectState::Failure => "Failure",
            JobGroupProjectState::Skipped => "Skipped",
            JobGroupProjectState::Canceled => "Canceled",
        };
        write!(f, "{}", value)
    }
}

impl FromStr for JobGroupProjectState {
    type Err = ProtocolError;

    fn from_str(value: &str) -> result::Result<Self, Self::Err> {
        match value.to_lowercase().as_ref() {
            "notstarted" => Ok(JobGroupProjectState::NotStarted),
            "inprogress" => Ok(JobGroupProjectState::InProgress),
            "success" => Ok(JobGroupProjectState::Success),
            "failure" => Ok(JobGroupProjectState::Failure),
            "skipped" => Ok(JobGroupProjectState::Skipped),
            "canceled" => Ok(JobGroupProjectState::Canceled),
            _ => Err(ProtocolError::BadJobGroupProjectState(value.to_string())),
        }
    }
}

impl Serialize for JobGroupProjectState {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self as u64 {
            0 => serializer.serialize_str("NotStarted"),
            1 => serializer.serialize_str("InProgress"),
            2 => serializer.serialize_str("Success"),
            3 => serializer.serialize_str("Failure"),
            4 => serializer.serialize_str("Skipped"),
            5 => serializer.serialize_str("Canceled"),
            _ => panic!("Unexpected enum value"),
        }
    }
}

impl Serialize for JobGroupProject {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut strukt = serializer.serialize_struct("job_group_project", 4)?;
        strukt.serialize_field("name", &self.get_name())?;
        strukt.serialize_field("ident", &self.get_ident())?;
        strukt.serialize_field("state", &self.get_state())?;
        strukt.serialize_field(
            "job_id",
            &self.get_job_id().to_string(),
        )?;
        strukt.end()
    }
}

impl Serialize for JobGroup {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut strukt = serializer.serialize_struct("job_group", 5)?;
        strukt.serialize_field("id", &self.get_id().to_string())?;
        strukt.serialize_field("state", &self.get_state())?;
        strukt.serialize_field("projects", &self.get_projects())?;
        strukt.serialize_field("created_at", &self.get_created_at())?;
        strukt.serialize_field(
            "project_name",
            &self.get_project_name(),
        )?;
        strukt.end()
    }
}

impl Serialize for JobGraphPackageReverseDependencies {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut strukt = serializer.serialize_struct(
            "job_graph_package_reverse_dependencies",
            3,
        )?;
        strukt.serialize_field("origin", &self.get_origin())?;
        strukt.serialize_field("name", &self.get_name())?;
        strukt.serialize_field("rdeps", &self.get_rdeps())?;
        strukt.end()
    }
}

impl Serialize for JobGraphPackageStats {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut strukt = serializer.serialize_struct("job_graph_package_stats", 2)?;
        strukt.serialize_field("plans", &self.get_plans())?;
        strukt.serialize_field("builds", &self.get_builds())?;
        strukt.serialize_field(
            "unique_packages",
            &self.get_unique_packages(),
        )?;
        strukt.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;

    #[test]
    fn test_ansi_stripping() {
        let mut log = JobLog::new();
        log.set_is_complete(false);
        log.set_start(0);
        log.set_stop(4);

        let lines = vec![
            "[1;33m» Importing origin key from standard log[0m",
            "[1;34m★ Imported secret origin key core-20160810182414.[0m",
            "[1;33m» Installing core/hab-backline[0m",
            "[1;32m↓ Downloading[0m core/hab-backline/0.23.0/20170511220008",
        ];

        let input_lines = lines.iter().map(|l| l.to_string());
        let content = RepeatedField::from_iter(input_lines);
        log.set_content(content);

        log.strip_ansi();

        let stripped_lines: Vec<String> = log.get_content()
            .into_iter()
            .map(|l| l.to_string())
            .collect();

        let expected = vec![
            "» Importing origin key from standard log",
            "★ Imported secret origin key core-20160810182414.",
            "» Installing core/hab-backline",
            "↓ Downloading core/hab-backline/0.23.0/20170511220008",
        ];
        assert_eq!(stripped_lines, expected);
    }

}
