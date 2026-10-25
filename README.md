# Isegoria

**Building a shared collection of quiz questions, with rules people can check.**

## Why this project exists

Imagine a quiz about how your country's institutions work. Someone has to choose
the topics, write the questions and decide which answers count as correct.
Those choices matter: a quiz can inform people, but it can also favour one
viewpoint or repeat a widely believed mistake.

Giving one authority control over the questions concentrates that power.
Letting the largest crowd decide everything can reproduce the same problem.
Isegoria explores how people could propose and review questions together, with
clear rules, protection for their identities and no money required to take part.

The longer-term aim is a network run by different operators. Today, the working
core is a prototype that reviews questions on a single computer. It makes the
review process explicit and checkable; it does not make a majority infallible.

## How it works, in plain words

1. **Someone writes a question.** They provide its context, possible answers and
   the answer they believe is correct.
2. **Other people answer it.** Reviewers are drawn at random from the registered
   participants. They are shown the question and options without the author's
   proposed answer. They may consult books, the Internet or AI, choose an answer,
   say "none of these is correct", or say "I don't know".
3. **The program applies the review rule.** Each reviewer counts equally. Enough
   agreement with the author's answer accepts the question; enough disagreement
   discards it. Divided answers and insufficient participation are separate
   outcomes. Only accepted questions enter the usable collection.

For example, suppose 12 reviewers answer a question about a town's institutions.
If 9 choose the author's answer, the question is accepted. If 8 do, the result is
divided. If 9 disagree with the author, it is discarded, even if they choose
different alternatives. These examples assume everyone gives an answer other
than "I don't know"; abstentions and missing answers are not counted. The
numbers are provisional, and the detailed rules cover smaller response counts.

A correction must be submitted as a new question. The program does not rewrite
an answer to match the most popular alternative.

## What matters in the design

- **Participation without a financial stake.** There is no token or money to put
  at risk. An author's past review outcomes affect how many questions they can
  propose later, not how much their answers count as a reviewer. Reviewers earn
  no reputation for agreeing with the majority.
- **Privacy without disposable identities.** The aim is to let a person participate
  under separate aliases for different roles, while preventing them from creating
  extra identities to gain influence or erase their history. The complete network
  still needs integration and testing before this is a real-world guarantee.
- **Rules that can be checked.** Given the recorded inputs, the calculation can be
  repeated to check the outcome. That checks whether the procedure was followed;
  judging the facts remains people's responsibility.

Agreement can still be wrong: reviewers may rely on the same mistaken source or
coordinate their answers. The software does not check facts or judge sources.
A collection can also be unbalanced even if its individual questions are accepted:
a civic quiz containing only tax questions would leave much of civic life out.

## Where things stand

- **Implemented:** the offline question-review process, including recording
  outcomes and updating authors' proposal limits. This first stage has been
  implemented and checked; its numerical settings remain experimental.
- **In development:** connecting that process across computers, towards a small
  experimental network run by independent operators. Network components exist,
  but the complete review process does not yet run between those computers.
- **Not yet available:** a public quiz service, automatic assembly of balanced
  quizzes, or a validated measure of what respondents know. This is a research
  prototype, not a system ready for everyday use.

## Learn more

- [Current rules and limits](docs/00-overview.md).
- [Development progress and next steps](https://github.com/dsmear/isegoria/blob/feat/phase2-runtime/docs/10-roadmap.md).
- [Technical documentation and research history](docs/README.md).
- [Code structure and testing](ARCHITECTURE.md).

## License

[EUPL-1.2](LICENSE).
