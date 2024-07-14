# Issues

This documents forms the basis of an approximately prioritised high level roadmap for Ducker.

The "issues" listed here are the aspirational set of features I have in mind, sorted into approximate priority/dependency order (I can't really add much in the way of modals until I've got my direction straight with modals).

Feel free to raise issues in github.

- Add tracing of some sort
- Automated test strategy; perhaps obviously there aren't any tests in the repo.  This is due to the way in which the project started, but probably needs to change sooner rather than later!
- Modals should use a general purpose trait object of some sort - preferably only one modal field per page, in a similar way to pages in the page manager
- Support for "forms" of scrolling stateful widgets (TODO - look out for ratatui libraries that already support this)
- Run Images - Given an image, run a new container; provide a form/modal to allow user to configure the container
- Tag Image - Allow a user to re-tag an image & push the re-tagged image
- Add extra detail to describe view - it is currently more representative of a PoC as it doesn't include much information not already in the table.
- CPU/network trace in detail page for container
- Callbacks should use closures instead of boilerplate-heavy structs
- Add filters to list pages
- vitepress docs page
