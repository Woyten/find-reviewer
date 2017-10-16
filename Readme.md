#Readme
## General intent of the application
Find-reviewer is a web application which manages a queue for code reviews.
The queue basically obeys the following rules: 
 * Every coder can be in the queue only once.
 * If the WIP limit is reached, the coder cannot be added to the queue but must do a review, first.

## Prerequisites
You need elm and rust on your machine in order to install find-reviewer.

## General setup
The setup is done using gradle. Run 'gradle release' to create a 'release'-directory in the root folder
of the project. The server can now be run by calling './find-reviewers' in this directory.

## Usage
Every user of the application is provided with a shared secret ("token") for the application in order to login.
Once the web application is called the user can paste the token into the application and the 
token will be automatically saved as a cookie. From now on the user can use the application to
request or perform code reviews.

## Configuration
The application has two configuration files:
 * find-reviewer.json
 * find-reviewer-users.json

The first is the generic configuration file which contains paramters like WIP limit or timeout rules.
The second file is the user database. The latter simply consists of a map which maps tokens to user
clear text user names.
