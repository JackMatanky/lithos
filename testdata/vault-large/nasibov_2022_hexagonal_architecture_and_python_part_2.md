---
title: nasibov_2022_hexagonal_architecture_and_python_part_2
uuid:
aliases:
  - "Hexagonal Architecture and Python - Part II: Domain, Application Services, Ports and Adapters"
  - "Hexagonal Architecture and Python: Part II, Domain, Application Services, Ports and Adapters"
  - hexagonal_architecture_and_python_part_ii_domain_application_services_ports_and_adapters
  - hexagonal_architecture_and_python_part_domain_application_services_ports_and_adapters
  - nasibov_2022_hexagonal_architecture_and_python_part_2
main_title: Hexagonal Architecture and Python
subtitle: "Part II, Domain, Application Services, Ports, and Adapters"
author:
  - "[[nasibov_zaur|Zaur Nasibov]]"
editor:
translator:
date_published: 2022-09-18
publisher:
url: "https://www.zaurnasibov.com/posts/2022/09/18/hexarch_di_python_part_2.html"
about: "Welcome to the second part of the article series, which cover principles of Hexagonal architecture, Dependency Injection, Domain-Driven Design and applies these all to Python and Django application design."
status: "undetermined"
type: "webpage"
file_class: "lib_webpage"
cssclasses:
date_created: 2025-10-11T15:38
date_modified: 2025-10-11T15:54
tags: "clippings"
---
# Hexagonal Architecture and Python - Part II: Domain, Application Services, Ports and Adapters

> [!webpage] Webpage Details
>
> - **Author**: `dv: this.file.frontmatter.author`
> - **Publisher**: `dv: this.file.frontmatter.publisher`
> - **Date Published**: `dv: this.file.frontmatter.date_published`
> - **Link**: `dv: elink(this.file.frontmatter.url, this.file.frontmatter.title)`
> - **About**: `dv: this.file.frontmatter.about`
>
> **Completed**::

---

![[hexagonal-python-2.png|Python logo in a hexagon with Roman II literal]]
- [Part I: Dependency Injection and componential architecture](https://zaurnasibov.com/posts/2021/10/30/hexarch_di_python_part_1.html)
- [Part II: Domain, Application Services, Ports and Adapters](https://zaurnasibov.com/posts/2022/09/18/hexarch_di_python_part_2.html)
- [Part III: Persistence, Transactions, Exceptions and The Final Assembly](https://zaurnasibov.com/posts/2022/12/31/hexarch_di_python_part_3.html)
- [Part IV: Lightweight integration tests](https://zaurnasibov.com/posts/2025/05/10/hexarch-python-part-4-lightweight-integration-tests.html)
- [The code](https://github.com/BasicWolf/hexagonal-architecture-django/tree/blog)

> [!Update]
>
> **February 2025**
>
> Originally, I said that the domain model is the first thing to put in code. Today, I always start with public interfaces, and in the case of RESTful services, the API. Though what I consider "the right way" has changed, I left this article almost intact, only commenting in a few places why API should be **coded** first.

## Intro

Now that you are familiar with the basic principles of Hexagonal architecture ([see part I](https://zaurnasibov.com/posts/2021/10/30/hexarch_di_python_part_1.html)) let's try implementing a Django-based application following these principles. I've chosen Django for this exercise to demonstrate that even an opinionated framework is not an obstacle for Hexagonal architecture. What about the other web frameworks, like FastAPI, Flask, AIOHTTP with SQLAchemy or a NoSQL data store?

Hexagonal architecture painlessly decouples the business logic from the technical details of HTTP communication, file system and database access, messaging and so on. You can swap Django with FastAPI, get rid of Django ORM and go with SQLAlchemy, and the business logic implementation remains the same!

The source code of the example is available at [BasicWolf/hexagonal-architecture-django](https://github.com/BasicWolf/hexagonal-architecture-django/tree/blog) Github repository.

> [!Tip]
>
> Clone the repository before reading further. The layered hexagonal architecture means deeply nested python packages. It's much easier to follow the article when the example code is available locally.

## Project Structure

A Django project is easily recognized by the top-level structure:

```txt
src/
  hexarch_project/             # Django application essentials: wsgi.py, urls.py, settings.py
  myapp/
    migrations/                # Django migrations
    apps.py                    # Django app configuration (creates dependencies container instance)
    dependencies_container.py  # Dependencies container
    models.py                  # Django DB models (imports models from SPI adapters)
    urls.py                    # Django urls mappings
    ⋮
    application/               # application code; the structure follows the principles of hexagonal architecture
```

Under the surface, the application *directory* structure is much deeper. From Python's perspective, all directories under application/ are **namespace-packages** i.e. there is no `__init__.py` in them. The namespace packages allow **the tests packages structure to reflect the application packages structure**.

The example application is structured as follows:

```txt
domain/            # Business domain models, services
  model/
  service/
port/              # API and SPI ports (interfaces)
  api/
  spi/
adapter/           # API and SPI ports implementation
  api/
    http/          # Django views and serializers
    messaging/
    ⋮
  spi/
    persistence/
      entity/      # Django models
      exceptions/  # Generic (django-independent) persistence exceptions
      repository/  # High-level persistence abstraction
    messaging/
    ⋮
service/           # Application services
```

Notice that Django is used only in adapters. Django Views are HTTP API adapters and Django Models are Persistence SPI adapters. The rest of the project is independent from the framework.

## Use Case: Upvote an Article

Now, let's take a look at an example use case. Imagine a web blogging platform in development. We want to add articles rating and let the users influence them.

We gathered with the end-users, platform experts, QA, and other stakeholders and drew some ideas:

1. Every article has a rating.
2. A user can change an article rating.
3. To change the article rating, a user either "upvotes" or "downvotes".
4. Users can vote for the article only if their "karma" (i.e. user rating) value is high enough, greater than 5.
5. A user can vote once per article.

A user story is born:

```txt
As a user of a blogging platform
I want to give my vote,
So that the article's rating changes.
```

## Where Do We Start?

That's a simple question, isn't it? Let's consider the options:

1. **Database**. It is important to integrate early with the downstream dependencies. The invisible bottlenecks could bring unpleasant surprises if integration is postponed till the last moment. Yet, a database is just storage and is usually located at the bottom layer of the application infrastructure. Do we really want to implement the database first and let it influence the application implementation?
2. **Public API**. Public API is the contract with the outer world. Its best design emerges when API consumers and producers collaborate. After the API specification is ready, the consumers and the producer (our service) can implement their part of the contract independently, and start integration as early as possible, even when no domain/business logic code exists yet!
	*We should always start by studying the domain, building shared understanding and deriving API from a detailed domain model. But when it comes to putting things in code, nowadays I tend to release API integration components as early as possible. In the early stages, I hard-code the returned values and focus on making sure that consumers can integrate. Only then do I start implementing the actual business logic.*
3. **Domain**. Starting with the domain model gives an advantage: we can test our **understanding** of the domain model, by expressing it in the code. Behaviour-driven development is essential at this stage. BDD brings techniques and tools to test the domain model code against the previously defined user stories. The problem is that we don't have the means to interact with the model yet. Which means that we have to go API first.
	*However, the article continues as originally put - the Domain model is implemented before API View. The flow of implementing the API first is quite different from domain-first.*

## The Domain

A discussion between the developers, users and domain experts gives a birth to a domain-specific language, where each term has a particular meaning. We call this language - [Ubiquitous language](https://www.martinfowler.com/bliki/UbiquitousLanguage.html) (UL).

In Hexagonal architecture, the domain layer encapsulates the business logic and business processes. On the code side, the UL terms are used in the names of classes, methods, and other code units. Thus, by looking at the code, you can always tell how it is related to the problem domain.

For example, a vote can be represented via the following enumeration \[[source](https://github.com/BasicWolf/hexagonal-architecture-django/blob/blog/src/myapp/application/domain/model/vote.py#L4)\]:

```python
# /src/myapp/application/domain/model/vote.py

class Vote(Enum):
    UP = 'up'
    DOWN = 'down'
```

Karma is an explicit type alias \[[source](https://github.com/BasicWolf/hexagonal-architecture-django/blob/blog/src/myapp/application/domain/model/karma.py)\]:

```python
# /src/myapp/application/domain/model/karma.py

Karma = NewType('Karma', int)
```

The most complex class of the domain is `VotingUser`, which represents a user that votes for an article. The `karma` value determines whether the user can vote. We also need to know whether the user has already voted to prevent repeat voting. Voting for an article produces a result:

![[4dee4275.png|uml diagram]]

```python
@dataclass
class VotingUser:
    id: UserId
    karma: Karma
    votes_for_articles: list[ArticleVote] = field(default_factory=list)

    def vote_for_article(
        self,
        article_id: ArticleId,
        vote: Vote
    ) -> VoteForArticleResult:
        if self._user_voted_for_article(article_id):
            return AlreadyVotedResult(article_id, self.id)

        if not self._karma_enough_for_voting():
            return InsufficientKarmaResult(user_id=self.id)

        ## IMPORTANT! The model state changes! ##
        self.votes_for_articles.append(
            ArticleVote(article_id, self.id, vote)
        )

        return SuccessfullyVotedResult(article_id, self.id, vote)

    ...
```

Notice that even the private methods `_user_voted_for_article()` and `_karma_enough_for_voting` follow the domain language. A fellow developer can easily map the code to the domain model and business rules.

You may wonder what `VoteForArticleResult` and `SuccessfullyVotedResult` are. Recall the basics of Hexagonal architecture from [Part I](https://zaurnasibov.com/posts/2021/10/30/hexarch_di_python_part_1.html):..

> Dependencies are directed from the outer layers to the inner centre.

`VoteForArticleResult` \[[source](https://github.com/BasicWolf/hexagonal-architecture-django/blob/blog/src/myapp/application/domain/model/vote_for_article_result.py#L10)\] is a domain data transfer object model. It carries the voting result from the innermost application layer - the Domain - to the outermost API adapter layer.

Writing tests for a domain model is straightforward since `VotingUser.vote_for_article(…)` return value is determined only by the `VotingUser` instance state and the method input values. With meaningful fixtures names, tests turn into simple scenarios, for example \[[source](https://github.com/BasicWolf/hexagonal-architecture-django/blob/blog/tests/test_myapp/application/domain/model/test_voting_user.py#L16)\]:

```python
# tests/test_myapp/application/domain/model/test_voting_user.py

def test_vote_for_article_twice_returns_already_voted_result(
    voting_user_who_has_voted: VotingUser,
    article_id_for_which_user_has_voted: ArticleId,
    a_vote: Vote,
    expected_already_voted_result: AlreadyVotedResult
):
    voting_result = voting_user_who_has_voted.vote_for_article(
        article_id_for_which_user_has_voted,
        a_vote
    )
    assert voting_result == expected_already_voted_result
```

Once the domain model is fully implemented, we switch our focus to its primary users - application services.

## Application Service: a Skeleton

Application services are the conductors that orchestrate processes and data flow in the application. An application service implements one or more related use cases and invokes all the necessary dependencies required to perform these use cases. The service implementation can start with a single return statement only:

```python
# /src/myapp/application/service/article_rating_service.py

class ArticleRatingService(
    VoteForArticleUseCase
):
    def vote_for_article(self, command: VoteForArticleCommand) -> VoteForArticleResult:
        return SuccessfullyVotedResult(
            command.article_id,
            command.user_id,
            command.vote
        )
```

This dummy implementation is good enough to echo the commands back, but much more needed to execute the use case. For example, where does the application service get a VotingUser?

## SPI Ports

In Hexagonal Architecture, an application service communicates with the outer world via Service Provider Interface (SPI) ports. The application service fetches the users by `user_id` and `article_id`. That can be expressed in a FindVotingUserPort interface as follows \[[source](https://github.com/BasicWolf/hexagonal-architecture-django/blob/blog/src/myapp/application/port/spi/find_voting_user_port.py#L8)\]:

```python
# /src/myapp/application/port/spi/find_voting_user_port.py

class FindVotingUserPort(Protocol):
    def find_voting_user(self, article_id: ArticleId, user_id: UserId) -> VotingUser:
        pass
```

We add `FindVotingUserPort` as a dependency to the application service. In practice, we add a respective field and a way to initialize it via constructor:

```python
# /src/myapp/application/service/article_rating_service.py

class ArticleRatingService(
    VoteForArticleUseCase
):
    _find_voting_user_port: FindVotingUserPort
    ...

    def __init__(
        self,
        find_voting_user_port: FindVotingUserPort,
        ...
    )
        self._find_voting_user_port = find_voting_user_port
```

Can you tell, what actual implementation is behind that interface? Is `VotingUser` found from a file? A database? Perhaps another HTTP endpoint? Or a hard-coded value? The application service does not care. It just makes a call:

```python
# /src/myapp/application/service/article_rating_service.py

class ArticleRatingService(...):
    def vote_for_article(self, command: VoteForArticleCommand) -> VoteForArticleResult:
        voting_user = self._find_voting_user_port.find_voting_user(
            command.article_id,
            command.user_id
        )
        ...
```

Another responsibility of the service is to persist the voting results.

It terms of [Domain-Driven Design](https://en.wikipedia.org/wiki/Domain-driven_design), `VotingUser` is an [Aggregate root](https://martinfowler.com/bliki/DDD_Aggregate.html). To update an article rating we have to persist a `VotingUser` as a whole. `SaveVotingUserPort` takes care of that \[[source](https://github.com/BasicWolf/hexagonal-architecture-django/blob/blog/src/myapp/application/port/spi/save_voting_user_port.py#L6)\]:

```python
# /src/myapp/application/port/spi/save_voting_user_port.py

class SaveVotingUserPort(Protocol):
    def save_voting_user(self, voting_user: VotingUser) -> VotingUser:
        raise pass
```

## Putting the Service Pieces together

Finally, `ArticleRatingService` has all the bits and pieces required to execute the use case \[[source](https://github.com/BasicWolf/hexagonal-architecture-django/blob/blog/src/myapp/application/service/article_rating_service.py#L16)\]:

```python
# /src/myapp/application/service/article_rating_service.py

class ArticleRatingService(
    VoteForArticleUseCase
):
    _find_voting_user_port: FindVotingUserPort
    _save_voting_user_port: SaveVotingUserPort

    ...

    def vote_for_article(self, command: VoteForArticleCommand) -> VoteForArticleResult:
        voting_user = self._find_voting_user_port.find_voting_user(
            command.article_id,
            command.user_id
        )

        voting_result = voting_user.vote_for_article(
            command.article_id,
            command.vote
        )

        match voting_result:
            case SuccessfullyVotedResult():
                self._save_voting_user_port.save_voting_user(voting_user)

        return voting_result
```

First, the service gets the required `VotingUser`. Next, the user votes for the article. Last, the service checks whether the user has successfully voted and persists the user state.

> [!Note]
>
> Do you remember that an application service is supposed to orchestrate the flow without any knowledge of its content? You may have noticed, that our application service does not fulfill this promise. The service controls the flow in `match voting_result`: block. I had to cheat here to make the code easier to follow and comprehend. One of the purer alternatives is the [Domain events](https://docs.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/domain-events-design-implementation) mechanism. It is a very interesting, but a huge topic, and unfortunately it falls out of the scope of this article.

## Test-driven Application Services Development

Testing application service differs from testing a domain model. Unlike a domain model, an application service has SPI dependencies, which should be replaced with test doubles in each test. That fact should not complicate the tests, though. It is possible to construct and explicitly pass a dependency test double and rely on default values for the rest of them.

For example, we test that the service persists the voting user \[[source](https://github.com/BasicWolf/hexagonal-architecture-django/blob/blog/tests/test_myapp/application/service/test_article_rating_service.py#L42)\]. The only dependency explicitly declared and passed to the service builder is `SaveVotingUserPortMock` test double. All other dependencies are provided by the `build_article_rating_service()` builder function:

```python
# /tests/test_myapp/application/service/test_article_rating_service.py

def test_voting_user_saved(
    self,
    vote_for_article_command: VoteForArticleCommand,
    saved_voting_user: VotingUser
):
    save_voting_user_port_mock = SaveVotingUserPortMock()
    article_rating_service = build_article_rating_service(
        save_voting_user_port=save_voting_user_port_mock
    )

    article_rating_service.vote_for_article(vote_for_article_command)

    assert save_voting_user_port_mock.saved_voting_user == saved_voting_user
```

The application service which implements `VoteForArticleUseCase` is ready. Next, we implement the adapter that invokes the use case.

## HTTP API

Let's start with the specification skeleton. Notice how the request and responses are derived from the domain model:

> [!Note]
>
> I deliberately omit the complete specification since the topic is out of this article's scope.

Django Rest Framework is the obvious choice to facilitate a RESTful endpoint implementation with Django. Beside the request and response processing, the logic fits into few lines of code \[[source](https://github.com/BasicWolf/hexagonal-architecture-django/blob/blog/src/myapp/application/adapter/api/http/article_vote_view.py#L29)\]:

```python
# /src/myapp/application/adapter/api/http/article_vote_view.py

class ArticleVoteView(APIView):

    def __init__(self, vote_for_article_use_case: VoteForArticleUseCase):
        self.vote_for_article_use_case = vote_for_article_use_case
        super().__init__()

    def post(self, request: Request) -> Response:
        vote_for_article_command = self._read_command(request)
        result = self.vote_for_article_use_case.vote_for_article(
            vote_for_article_command
        )
        return self._build_response(result)

    ...
```

The intentions are:

1. Accept the HTTP request, deserialize and validate the request data.
2. **Invoke the use case**.
3. Serialize the result and render the response.

The view has one dependency: an object which implements `VoteForArticleUseCase` protocol \[[source](https://github.com/BasicWolf/hexagonal-architecture-django/blob/blog/src/myapp/application/port/api/vote_for_article_use_case.py#L9)\]:

```python
# /src/myapp/application/port/api/vote_for_article_use_case.py

class VoteForArticleUseCase(Protocol):
   def vote_for_article(self, command: VoteForArticleCommand) -> VoteForArticleResult:
       pass
```

As we already know, the `ArticleRatingService` has implemented this protocol as is ready to be wired with the controller.

Testing a HTTP controller is no different from testing an application service. Every test injects a tuned double of the `VoteForArticleUseCase` dependency and asserts the expected state or behavior. This makes the view testing times more lightweight compared to the traditional, spin-it-all-up Django app testing.

For example, how to test a scenario, where a user tries to vote twice in a row? \[[source](https://github.com/BasicWolf/hexagonal-architecture-django/blob/blog/tests/test_myapp/application/adapter/api/http/test_article_vote_view.py#L25)\]

```python
# /tests/test_myapp/application/adapter/api/http/test_article_vote_view.py

def test_user_votes_for_the_same_article_returns_conflict(
    arf: APIRequestFactory
):
    ## This is *the* view we are testing
    article_vote_view = ArticleVoteView.as_view(
        ## we are injecting a stub which always
        ## returns AlreadyVotedResult (see below)
        vote_for_article_use_case=VoteForArticleUseCaseAlreadyVotedStub()
    )

    ## A valid article vote is POSTed
    response: Response = article_vote_view(
        arf.post(
            '/article_vote',
            {
                'user_id': UserId(UUID('a3854820-0000-0000-0000-000000000000')),
                'article_id': ArticleId(UUID('dd494bd6-0000-0000-0000-000000000000')),
                'vote': Vote.UP.value
            },
            format='json'
        )
    )

    ## But the result is HTTP 409, as defined in the specification
    assert response.status_code == HTTPStatus.CONFLICT
    assert response.data == {
        'status': 409,
        'detail': "User \"a3854820-0000-0000-0000-000000000000\" has already voted"
                  " for article \"dd494bd6-0000-0000-0000-000000000000\"",
        'title': "Cannot vote for an article"
    }
```

And here is the `VoteForArticleUseCaseAlreadyVotedStub` \[[source](https://github.com/BasicWolf/hexagonal-architecture-django/blob/blog/tests/test_myapp/application/adapter/api/http/test_article_vote_view.py#L148)\]:

```python
# /tests/test_myapp/application/adapter/api/http/test_article_vote_view.py

class VoteForArticleUseCaseAlreadyVotedStub(VoteForArticleUseCase):
    def vote_for_article(self, command: VoteForArticleCommand) -> VoteForArticleResult:
        return AlreadyVotedResult(
            user_id=command.user_id,
            article_id=command.article_id
        )
```

Please pause for a moment. Does it take much effort to grok the test? Did you notice that the test does not interact with rest of the application? Did you also notice that it takes only five lines of code (three, if you put the `return` on a single line!) to mock "the rest of the application"? The responsibilities are clearly decoupled, and there is no need to set up a database or *any* other service to test an HTTP endpoint.

## What's next

This concludes Part II of the article series about Hexagonal Architecture and Python and Django. Part III will discuss how to use Django Models in SPIs, manage database transactions and put all the application pieces together. Stay tuned!

## Acknowledgments

I would like to express my very great appreciation to [Jere "Urokhtor" Teittinen](https://jereteittinen.info/) and [Jarkko "jmp" Piiroinen](https://jarkko.org/) for reviewing the article and helping to improve it!
