---
title: nasibov_2021_hexagonal_architecture_and_python_part_1
uuid:
aliases:
  - "Hexagonal Architecture and Python - Part I: Dependency Injection and Componential Architecture"
  - "Hexagonal Architecture and Python: Part I, Dependency Injection and Componential Architecture"
  - hexagonal_architecture_and_python_part_i_dependency_injection_and_componential_architecture
  - hexagonal_architecture_and_python_dependency_injection_and_componential_architecture
  - nasibov_2021_hexagonal_architecture_and_python_part_1
main_title: Hexagonal Architecture and Python
subtitle: "Part I, Dependency Injection and Componential Architecture"
author:
  - "[[nasibov_zaur|Zaur Nasibov]]"
editor:
translator:
date_published: 2021-10-30
publisher:
url: "https://www.zaurnasibov.com/posts/2021/10/30/hexarch_di_python_part_1.html"
about: "Welcome to articles series which cover the principles of Hexagonal architecture, talks of Dependency Injection, and its usage in these to Python and Django application design."
status: "undetermined"
type: "webpage"
file_class: "lib_webpage"
cssclasses:
date_created: 2025-10-11T15:23
date_modified: 2025-10-11T15:39
tags:
---
# Hexagonal Architecture and Python - Part I: Dependency Injection and Componential Architecture

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

![[hexagonal-python.png|Python logo in a hexagon]]
- [Part I: Dependency Injection and componential architecture](https://zaurnasibov.com/posts/2021/10/30/hexarch_di_python_part_1.html)
- [Part II: Domain, Application Services, Ports and Adapters](https://zaurnasibov.com/posts/2022/09/18/hexarch_di_python_part_2.html)
- [Part III: Persistence, Transactions, Exceptions and The Final Assembly](https://zaurnasibov.com/posts/2022/12/31/hexarch_di_python_part_3.html)
- [Part IV: Lightweight integration tests](https://zaurnasibov.com/posts/2025/05/10/hexarch-python-part-4-lightweight-integration-tests.html)

Time flies awfully fast! Two and a half years ago I left the world of Django and found myself in the world of Spring Boot, Kotlin and Java. It was a genuine cultural shock. An extensive amount of new knowledge bombarded my brains. Sometimes it was so furious, that I wanted to run back to the beloved and bytewise familiar Python ecosystem. Inversion of Control (IoC) was the hardest topic to digest. Automated Dependency Injection (DI) felt like black magic, compared to Django's direct approach. Spring Boot behemoth framework consumed me in nightmares. But all the effort was worth it.

We designed and implemented the application following Hexagonal architecture rules. And the final challenge was getting rid of the old "implement a backlog of features" habit in place of Domain-Driven Design (DDD).

Our product is rapidly growing in size and complexity. Still, it is easy to maintain, support and develop - thanks to the solid foundation. The code is expressive and comprehensible. The components are easily interchangeable. By all means this product is better than anything previously written by the team members.

I look behind and see all the gaps in my experience which did not allow solving business problem as elegantly. Dear fellow Pythonista, I hope this short articles series about Hexagonal architecture would help you to achieve the same without going through my struggle.

## Dependency Injection

You know what Dependency Injection (DI) is, don't you? Sure you do, even if you can't recall the explicit definition. Let's see what are the pros and cons of this approach (a pattern, if you prefer).

Imagine that we need a function which sends ALARM messages to a message bus. The first iteration is:

```python
from my_cool_messaging_library import get_message_bus()

def send_alert(message: str):
    message_bus = get_message_bus()
    message_bus.send(topic='alert', message=message)
```

Is there anything wrong with `send_alert()` function? It depends upon `message_bus` object, but this dependency is hidden from the caller. What if you'd like to use another message bus? How about the level of magic required to test this function? Did I just hear `mock.patch(…)`? Ladies and gentlemen, this is going south, let's try a different way:

```python
from my_cool_messaging_library import MessageBus

def send_alert(message_bus: MessageBus, message: str):
    message_bus.send(topic='alert', message=message)
```

This small change in function signature is a big change of paradigm. The caller sees that `send_alert()` function **depends** upon MessageBus object (viva type annotations!). All implicit mocking bells and whistles are gone in favour of explicit and clean code. Sounds too good to be true? Have a look:

```python
def test_send_alert_sends_message_to_alert_topic()
    message_bus_mock = MessageBusMock()
    send_alert(message_bus_mock, "Hacking attempt detected!")

    assert message_bus_mock.sent_to_topic == 'alert'
    assert message_bus_mock.sent_message == "Hacking attempt detected!"

class MessageBusMock(MessageBus):
    def send(self, topic, message):
        self.sent_to_topic = topic
        self.sent_message = message
```

But doesn't this mean that we have to pass an instance of MessageBus to `send_alert()` function on each call? Isn't that cumbersome?

```python
send_alert(get_message_bus(), "Stackoverflow is down")
```

Let's try solving this problem by means of OOP:

```python
class AlertDispatcher:
    _message_bus: MessageBus

    def __init__(self, message_bus: MessageBus):
        self._message_bus = message_bus

    def send(message: str):
        self._message_bus.send(topic='alert', message=message)

alert_dispatcher = AlertDispatcher(get_message_bus())
alert_dispatcher.send("Oh no, yet another dependency!")
```

Now `AlertDispatcher` class **depends** on an object of type MessageBus. We **inject** this dependency when creating an `AlertDispatcher` object by passing the dependency into constructor. We have **wired** (not coupled!) the object and its dependency.

At this point the focus switches from `message_bus` to `alert_dispatcher`. This **component** may be required in different parts of the application. Which means that there should be a global context which holds and provides the object. Let's first discuss the nature of components and components wiring.

## Componential Architecture

We didn't emphasize dependencies types while speaking of dependency injection. But you might have guessed that MessageBus is just an abstraction, an interface or a protocol.[^1] Somewhere the application defines:

```python
class MessageBus(typing.Protocol):
    def send(topic: str, message: str):
        pass
```

There is also a simple implementation of MessageBus in the project. It stores the incoming messages in a list:

```python
class MemoryMessageBus(MessageBus):
    sent_messages = []

    def send(topic: str, messagge: str):
        self.sent_messages.append(topic, message)
```

In the same manner, an abstract use case scenario is decoupled from a business-driven implementation:

```python
# An abstract use case
class DispatchAlertUseCase(typing.Protocol):
    def dispatch_alert(message: str):
        pass
```

Next, let's add a controller which accepts HTTP requests and invokes DispatchAlertUseCase:

```python
class ChatOpsController:
    ...
    def __init__(self, dispatch_alert_use_case: DispatchAlertUseCase):
        self._dispatch_alert_use_case = dispatch_alert_use_case

    @post('/alert)
    def alert(self, message: Message):
        self._dispatch_alert_use_case.dispatch_alert(message)
        return HTTP_ACCEPTED
```

Finally, let's connect all the pieces together:

```python
from my_favourite_http_framework import http_server

def main():
    message_bus = MemoryMessageBus()
    alert_dispatcher_service = AlertDispatcherService(message_bus)
    chat_opts_controller = ChatOpsController(alert_dispatcher_service)

    http_server.start()
```

How would a rational and clear-minded developer react to this? She would call it overengineered and overcomplicated, no less! Which is indeed true. On the first glance, everything above fits into a short HTTP handler:

```python
@post('/alert)
def alert(message: Message):
    bus = MemoryMessageBus()
    bus.send(topic='alert', message=message)
    return HTTP_ACCEPTED
```

Is it short and simple? Absolutely! Is it maintainable? Hardly. But why? Because the components are strongly coupled in the code. By blending everything in a single function we tightly couple domain workflow and message bus implementation. And that's half the trouble. The worst part is that we *melted and buried business logic in technical details*. Don't get me wrong, such code has the right to exist. Yet its existence in a rapidly growing application will soon end up in a maintenance hell.

Back to the componential architecture. What are the advantages?

- Components are **isolated** and are not directly dependent. Instead they are **wired via abstractions**.
- Every component works in certain boundaries and **has a single responsibility**.
- This means that components are immensely testable: either in full isolation or in a combination using test doubles. There is no need to explain that testing isolated parts of a program is easier compared to testing it as a whole. Your TDD approach improves from inaudible "well, we do tests…" to sonorous "tests-driven and test-first development".
- It is easy to substitute components, thanks to abstract dependencies. In the example above `MemoryMessageBus` could be replaced with `DbMessageBus`, `FileMessageBus` or anything else. The caller of `message_bus.send(…)` should not care.

Suddenly it dawns upon you: "That sounds like… SOLID?" Hell yeah! It is almost what Uncle Bob would call a [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html). I encourage you to read his article, before moving towards our end goal - Hexagonal architecture.

## Architecture is about Intent

One of my favourite Uncle Bob quotes on software architecture is "Architecture is about intent".

What do you see on this screenshot?

![[django_project.png|Typical Django project]]

No wonder if you recognized a "typical Django application". Brilliant! Can you also tell what does this application do? If you can, my sincere congratulations - you are level 80 telepathist. Personally I have no clue whatsoever - that is a screenshot of a random Django application from Github.

Robert Martin develops the idea [further](https://www.youtube.com/watch?v=WpkDN78P884). Take a look at a floor architecture plan and guess what the building is intended for:

![[library_floor_paln.jpg|Typical Django project]] Answer

That is a floor plan of [Oodi Library](https://en.wikipedia.org/wiki/Helsinki_Central_Library_Oodi) in Helsinki.

I hope this tiny puzzle was easy to solve and you got the main idea: architecture should meet us at the gate, literally after git clone. Isn't it great when the source code is organized in such way that the purpose and the meaning of each file, class, function and any other object lies on the surface?

## Hexagonal Architecture of Ports and Adapters

"We use the Hexagonal architecture of ports and adapters" - how we start describing the architecture application to the new team members. It follows by showing a weird Cthulhu-like picture:

![[hexagon.png|Hexagonal architecture]]

Alistair Cockurn, the inventor of "Hexagonal architecture" term explains that "hexagon" is not strictly necessary:

> The hexagon is not a hexagon because the number six is important, but rather to allow the people doing the drawing to have room to insert ports and adapters as they need, not being constrained by a one-dimensional layered drawing.
>
> The term ''hexagonal architecture'' comes from this visual effect.
>
> — [Alistair Cockburn](https://alistair.cockburn.us/hexagonal-architecture/)

You may also recall the terms like "Onion architecture" or "Ports and Adapters" mentioned in Uncle Bob's ["Clean archcitecture"](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html) article. All these terms describe a way to organize application architectecture in layers, with dependencies directed to the center. (We noticed though, that *hexagon* and *ports and adapters* are much simpler to imagine and explain compared to more abstract terms like "clean" or "onion").

The outer layer of the application - *adapters* - interacts with the outer world. The inner layer - *domain* and *domain services* - contains the business logic. The connecting layer in between is *application services*. The components in each layer are designed to have low coupling and high cohesion.[^2]

**Domain** layer - is the heart of the application. The business rules are defined here. The names of classes, methods, functions, constants and other objects resemble those of the problem domain.

**Application Programming Interface (API) adapters** are the components which convey commands and queries from the outer world to the application through *API ports*. Hence, **API ports** are interfaces through which the application is controlled and queried. In the *Componential architecture* described above, the is an *API port* and is an *API adapter*.

**Service Provider Interface (SPI) adapters** are the components which convey the application commands and queries to the outer world. Hence, **SPI ports** are interfaces through which commands and queries are passed to *SPI adapters*. In the *Componential architecture*, the is an *SPI port* and its implementation is an *SPI adapter*.

**Application services** are the conductors which glue domain and ports performing use case scenarios. They are supposed to orchestrate the whole transaction, e.g. load data for the domain to use, call functions which perform domain logic, persist the results and publish events.

There is one important rule to remember about hexagonal architecture: " *Dependencies are directed from the outer layers to the inner center.*" In practice this means that *adapters are aware of domain objects, but domain should not know of adapters or ports*. It's also important to clarify that since adapters belong to the *Adapters layer*, they may call each-other directly. The cases for this are:

- *API adapter calls SPI adapter*. - Think of HTTP GET requests which end up querying a database: It doesn't make sense to reach database through the domain layer making tons of objects mappings on the way. It's much more efficient for a controller to call the specific *SPI port* directly.
- *SPI adapter calls SPI adapter.* - Think of an database adapter which calls other adapters and aggregates all the results before returning it.
- *API adapter calls API adapter.* - Probably the least useful. Handy, if you need to redirect calls from one API adapter to the other.
- *SPI adapter calls API adapter* - AVOID. Such calls is unnecessary and creates call loops in the application.

And… THAT IS IT! The basic principles of Hexagonal architecture of ports and adapters are surprisingly simple. This kind of architecture works well in application with complex problem domain. But it is an overkill, if everything you need is an "HTTP interface for a database".

We are now ready to dive into building a Hexagonal architecture -based Django applications. Stay tuned for part II.

## References

[^1]: Though [PEP-544](https://www.python.org/dev/peps/pep-0544/) Protocols are about structural subtying. Another option to mimic interfaces is to either use abstract classes with abstract methods.
[^2]: Cohesion and coupling:
	- [Devopedia: Cohesion vs. Coupling](https://devopedia.org/cohesion-vs-coupling)
	- [Cohesion and Coupling: the difference](https://enterprisecraftsmanship.com/posts/cohesion-coupling-difference/)
