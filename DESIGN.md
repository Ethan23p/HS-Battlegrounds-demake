# HS-Battlegrounds-demake — Design

## Process

This document is built by interview.

**Claude's role:** informational interviewer, tasked with asking an extremely minimal
question (the creation of which Ethan guides) which extracts from the User a sufficient
answer.

Before each of the following steps, Claude prints them verbatim in the chat to the User.

**Step 0**
Establish a file in the project root which contains two sections, if it doesn't exist:
Vision and Features (features = the 'deltas').

**Step 1**
Choose a topic from: vision + each feature; each topic should be covered only once.
Then draft an extremely minimal question which straightforwardly prompts the User to
respond naturally and in detail about the topic.

**Step 2**
Assume that the User's response was sufficient.
Place only the question and the User's response, formatted nicely, in the document.

Repeat.

---

## Vision

**Q: What are you making, and why that?**

> I'd like to make a minimal demake of Hearthstone: Battlegrounds - just a gameplay
> vertical slice - with a handful of departures which should end up feeling like a
> fairly distinct autobattler game.

---

## Features

**Order:** beats (done) → targeting → death-resolution timing → keywords → abilities-as-data.

### Feature 1

**Q: What's the first departure from Battlegrounds you want to talk about — what is it, and how does it work?**

> In Battlegrounds, the combat phase proceeds one step at a time - a step is a container
> containing a single interaction, which could be effectively anything (a unit action, an
> effect, hero power) - and many interactions spawn steps which have to resolve in place
> before the next step can be taken. These steps are always synchronous transactions and
> usually involve 1 origin entity and 1 target entity - except when they don't.
>
> Instead of Battlegrounds' 'step' this app has beats which are just like steps except
> they support concurrent transactions between arbitrary entities - in a BG's step, an
> interaction has to resolve before the next can begin, but this app's beats contain any
> arbitrary number of interactions we choose; nonetheless, most beats involve one
> interaction.
