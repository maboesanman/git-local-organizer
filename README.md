# git-local-organizer

Opinionated git source management

Configure a Source directory in your user folder, and all repos will be installed in an organized way in that folder.

For example calling `git-local-organizer https://github.com/maboesanman/git-local-organizer.git` will clone the repo into `~/Source/github.com/maboesanman/git-local-organizer`

In addition to organizing, when cloning a fork the original will be places in its canonical location, and a symlink will be created, as well as remotes being created in the forked repo.

Currently only unix is supported and only github (including enterprise) for fork resolution. The fork symlinking can be opted out of.
