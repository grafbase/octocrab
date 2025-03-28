//! The pull request API.

mod comment;
mod create;
mod update;
mod list;
mod merge;

use snafu::ResultExt;

use crate::{Octocrab, Page};

pub use self::{create::CreatePullRequestBuilder, update::UpdatePullRequestBuilder, list::ListPullRequestsBuilder};

/// A client to GitHub's pull request API.
///
/// Created with [`Octocrab::pulls`].
pub struct PullRequestHandler<'octo> {
    crab: &'octo Octocrab,
    owner: String,
    repo: String,
    media_type: Option<crate::params::pulls::MediaType>,
}

impl<'octo> PullRequestHandler<'octo> {
    pub(crate) fn new(crab: &'octo Octocrab, owner: String, repo: String) -> Self {
        Self {
            crab,
            owner,
            repo,
            media_type: None,
        }
    }

    /// Set the media type for this request.
    /// ```no_run
    /// # async fn run() -> octocrab::Result<()> {
    /// let pr = octocrab::instance()
    ///     .pulls("owner", "repo")
    ///     .media_type(octocrab::params::pulls::MediaType::Full)
    ///     .get(404)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn media_type(mut self, media_type: crate::params::pulls::MediaType) -> Self {
        self.media_type = Some(media_type);
        self
    }

    /// Checks if a given pull request has been merged.
    /// ```no_run
    /// # async fn run() -> octocrab::Result<()> {
    /// # let octocrab = octocrab::Octocrab::default();
    /// octocrab.pulls("owner", "repo").is_merged(101).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_merged(&self, pr: u64) -> crate::Result<bool> {
        let route = format!(
            "repos/{owner}/{repo}/pulls/{pr}/merge",
            owner = self.owner,
            repo = self.repo,
            pr = pr
        );
        let response = self
            .crab
            ._get(self.crab.absolute_url(route)?, None::<&()>)
            .await?;

        Ok(response.status() == 204)
    }

    /// Update the branch of a pull request.
    ///
    /// ```no_run
    /// # async fn run() -> octocrab::Result<()> {
    /// # let octocrab = octocrab::Octocrab::default();
    /// octocrab.pulls("owner", "repo").update_branch(101).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_branch(&self, pr: u64) -> crate::Result<bool> {
        let route = format!(
            "repos/{owner}/{repo}/pulls/{pr}/update-branch",
            owner = self.owner,
            repo = self.repo,
            pr = pr
        );
        let response = self
            .crab
            ._put(self.crab.absolute_url(route)?, None::<&()>)
            .await?;

        Ok(response.status() == 202)
    }

    /// Get's a given pull request with by its `pr` number.
    /// ```no_run
    /// # async fn run() -> octocrab::Result<()> {
    /// let pr = octocrab::instance().pulls("owner", "repo").get(101).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, pr: u64) -> crate::Result<crate::models::pulls::PullRequest> {
        let url = format!(
            "repos/{owner}/{repo}/pulls/{pr}",
            owner = self.owner,
            repo = self.repo,
            pr = pr
        );

        self.http_get(url, None::<&()>).await
    }

    /// Get's a given pull request's `diff`.
    /// ```no_run
    /// # async fn run() -> octocrab::Result<()> {
    /// let diff = octocrab::instance().pulls("owner", "repo").get_diff(101).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_diff(&self, pr: u64) -> crate::Result<String> {
        let route = format!(
            "repos/{owner}/{repo}/pulls/{pr}",
            owner = self.owner,
            repo = self.repo,
            pr = pr
        );

        let request = self
            .crab
            .client
            .get(self.crab.absolute_url(route)?)
            .header(reqwest::header::ACCEPT, crate::format_media_type("diff"));

        let response = crate::map_github_error(self.crab.execute(request).await?).await?;

        response.text().await.context(crate::error::HttpSnafu)
    }

    /// Get's a given pull request's patch.
    /// ```no_run
    /// # async fn run() -> octocrab::Result<()> {
    /// let diff = octocrab::instance().pulls("owner", "repo").get_patch(101).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_patch(&self, pr: u64) -> crate::Result<String> {
        let route = format!(
            "repos/{owner}/{repo}/pulls/{pr}",
            owner = self.owner,
            repo = self.repo,
            pr = pr
        );

        let request = self
            .crab
            .client
            .get(self.crab.absolute_url(route)?)
            .header(reqwest::header::ACCEPT, crate::format_media_type("patch"));

        let response = crate::map_github_error(self.crab.execute(request).await?).await?;

        response.text().await.context(crate::error::HttpSnafu)
    }

    /// Create a new pull request.
    ///
    /// - `title` — The title of the new pull request.
    /// - `head` — The name of the branch where your changes are implemented.
    ///   For cross-repository pull requests in the same network, namespace head
    ///   with a user like this: `username:branch`.
    /// - `base` — The name of the branch you want the changes pulled into. This
    ///   should be an existing branch on the current repository. You cannot
    ///   submit a pull request to one repository that requests a merge to a
    ///   base of another repository.
    /// ```no_run
    /// # async fn run() -> octocrab::Result<()> {
    /// # let octocrab = octocrab::Octocrab::default();
    /// let pr = octocrab
    ///     .pulls("owner", "repo")
    ///     .create("title", "head", "base")
    ///     .body("hello world!")
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create(
        &self,
        title: impl Into<String>,
        head: impl Into<String>,
        base: impl Into<String>,
    ) -> create::CreatePullRequestBuilder<'octo, '_> {
        create::CreatePullRequestBuilder::new(self, title, head, base)
    }

    /// Update a new pull request.
    ///
    /// - `pull_number` — pull request number.
    /// ```no_run
    /// # async fn run() -> octocrab::Result<()> {
    /// # let octocrab = octocrab::Octocrab::default();
    /// let pr = octocrab
    ///     .pulls("owner", "repo")
    ///     .update(1)
    ///     .body("hello world!")
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn update(
        &self,
        pull_number: u64,
    ) -> update::UpdatePullRequestBuilder<'octo, '_> {
        update::UpdatePullRequestBuilder::new(self, pull_number)
    }

    /// Creates a new `ListPullRequestsBuilder` that can be configured to filter
    /// listing pulling requests.
    /// ```no_run
    /// # async fn run() -> octocrab::Result<()> {
    /// # let octocrab = octocrab::Octocrab::default();
    /// use octocrab::params;
    ///
    /// let page = octocrab.pulls("owner", "repo").list()
    ///     // Optional Parameters
    ///     .state(params::State::Open)
    ///     .head("master")
    ///     .base("branch")
    ///     .sort(params::pulls::Sort::Popularity)
    ///     .direction(params::Direction::Ascending)
    ///     .per_page(100)
    ///     .page(5u32)
    ///     // Send the request
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list(&self) -> list::ListPullRequestsBuilder {
        list::ListPullRequestsBuilder::new(self)
    }

    /// Lists all of the `Review`s associated with the pull request.
    /// ```no_run
    /// # async fn run() -> octocrab::Result<()> {
    /// let reviews = octocrab::instance().pulls("owner", "repo").list_reviews(101).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_reviews(&self, pr: u64) -> crate::Result<Page<crate::models::pulls::Review>> {
        let url = format!(
            "repos/{owner}/{repo}/pulls/{pr}/reviews",
            owner = self.owner,
            repo = self.repo,
            pr = pr
        );

        self.http_get(url, None::<&()>).await
    }

    /// Request a review from users or teams.
    /// ```no_run
    /// # async fn run() -> octocrab::Result<()> {
    /// let review = octocrab::instance().pulls("owner", "repo")
    ///    .request_reviews(101, ["user1".to_string(), "user2".to_string()], ["team1".to_string(), "team2".to_string()])
    ///  .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn request_reviews(
        &self,
        pr: u64,
        reviewers: impl Into<Vec<String>>,
        team_reviewers: impl Into<Vec<String>>,
    ) -> crate::Result<crate::models::pulls::Review> {
        let url = format!(
            "repos/{owner}/{repo}/pulls/{pr}/requested_reviewers",
            owner = self.owner,
            repo = self.repo,
            pr = pr
        );

        let mut map = serde_json::Map::new();
        map.insert("reviewers".to_string(), reviewers.into().into());
        map.insert("team_reviewers".to_string(), team_reviewers.into().into());

        self.crab.post(url, Some(&map)).await
    }

    /// List all `FileDiff`s associated with the pull request.
    /// ```no_run
    /// # async fn run() -> octocrab::Result<()> {
    /// let files = octocrab::instance().pulls("owner", "repo").list_files(101).await?;
    /// # Ok(())
    /// # }
    pub async fn list_files(&self, pr: u64) -> crate::Result<Page<crate::models::pulls::FileDiff>> {
        let url = format!(
            "repos/{owner}/{repo}/pulls/{pr}/files",
            owner = self.owner,
            repo = self.repo,
            pr = pr
        );

        self.http_get(url, None::<&()>).await
    }

    /// Creates a new `ListCommentsBuilder` that can be configured to list and
    /// filter `Comments` for a particular pull request. If no pull request is
    /// specified, lists comments for the whole repo.
    /// ```no_run
    /// # async fn run() -> octocrab::Result<()> {
    /// # let octocrab = octocrab::Octocrab::default();
    /// use octocrab::params;
    ///
    /// let page = octocrab.pulls("owner", "repo").list_comments(Some(5))
    ///     // Optional Parameters
    ///     .sort(params::pulls::comments::Sort::Created)
    ///     .direction(params::Direction::Ascending)
    ///     .per_page(100)
    ///     .page(5u32)
    ///     .since(chrono::Utc::now() - chrono::Duration::days(1))
    ///     // Send the request
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_comments(&self, pr: Option<u64>) -> comment::ListCommentsBuilder {
        comment::ListCommentsBuilder::new(self, pr)
    }

    /// Creates a new `MergePullRequestsBuilder` that can be configured used to
    /// merge a pull request.
    /// ```no_run
    /// # async fn run() -> octocrab::Result<()> {
    /// # let octocrab = octocrab::Octocrab::default();
    /// use octocrab::params;
    ///
    /// let page = octocrab.pulls("owner", "repo").merge(20)
    ///     // Optional Parameters
    ///     .title("cool title")
    ///     .message("a message")
    ///     // Won't merge of the HEAD commit of the PR branch is not the same
    ///     .sha("0123456")
    ///     // The method to use when merging, will default to `Merge`
    ///     .method(params::pulls::MergeMethod::Squash)
    ///     // Send the request
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn merge(&self, pr: u64) -> merge::MergePullRequestsBuilder {
        merge::MergePullRequestsBuilder::new(self, pr)
    }
}

impl<'octo> PullRequestHandler<'octo> {
    pub(crate) async fn http_get<R, A, P>(
        &self,
        route: A,
        parameters: Option<&P>,
    ) -> crate::Result<R>
    where
        A: AsRef<str>,
        P: serde::Serialize + ?Sized,
        R: crate::FromResponse,
    {
        let mut request = self.crab.client.get(self.crab.absolute_url(route)?);

        if let Some(parameters) = parameters {
            request = request.query(parameters);
        }

        if let Some(media_type) = self.media_type {
            request = request.header(
                reqwest::header::ACCEPT,
                crate::format_media_type(&media_type.to_string()),
            );
        }

        R::from_response(crate::map_github_error(self.crab.execute(request).await?).await?).await
    }

    pub(crate) async fn http_post<R, A, P>(&self, route: A, body: Option<&P>) -> crate::Result<R>
    where
        A: AsRef<str>,
        P: serde::Serialize + ?Sized,
        R: crate::FromResponse,
    {
        let mut request = self.crab.client.post(self.crab.absolute_url(route)?);

        request = self.build_request(request, body);

        R::from_response(crate::map_github_error(self.crab.execute(request).await?).await?).await
    }

    pub(crate) async fn http_put<R, A, P>(&self, route: A, body: Option<&P>) -> crate::Result<R>
    where
        A: AsRef<str>,
        P: serde::Serialize + ?Sized,
        R: crate::FromResponse,
    {
        let mut request = self.crab.client.put(self.crab.absolute_url(route)?);

        request = self.build_request(request, body);

        R::from_response(crate::map_github_error(self.crab.execute(request).await?).await?).await
    }

    pub(crate) async fn http_patch<R, A, P>(&self, route: A, body: Option<&P>) -> crate::Result<R>
    where
        A: AsRef<str>,
        P: serde::Serialize + ?Sized,
        R: crate::FromResponse,
    {
        let mut request = self.crab.client.patch(self.crab.absolute_url(route)?);

        request = self.build_request(request, body);

        R::from_response(crate::map_github_error(self.crab.execute(request).await?).await?).await
    }

    fn build_request<P>(
        &self,
        mut request: reqwest::RequestBuilder,
        body: Option<&P>,
    ) -> reqwest::RequestBuilder
    where
        P: serde::Serialize + ?Sized,
    {
        if let Some(body) = body {
            request = request.json(body);
        }

        if let Some(media_type) = self.media_type {
            request = request.header(
                reqwest::header::ACCEPT,
                crate::format_media_type(&media_type.to_string()),
            );
        }

        request
    }
}
