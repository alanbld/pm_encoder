"""
Concrete RepoSource implementations.

v1.6.0:
    - ClonedRepoSource: Shallow git clone to temp directory

v1.7.0+ (planned):
    - GitHubAPISource: Stream from GitHub API
    - TarballSource: Process release archives
    - InMemorySource: For testing
"""

from .cloned import ClonedRepoSource

__all__ = ["ClonedRepoSource"]
