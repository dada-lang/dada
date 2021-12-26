[
    Diagnostic {
        range: Range {
            start: Position {
                line: 2,
                character: 7,
            },
            end: Position {
                line: 2,
                character: 10,
            },
        },
        severity: Some(
            Error,
        ),
        code: None,
        source: None,
        message: "already have a class named `Foo`",
        related_information: Some(
            [
                DiagnosticRelatedInformation {
                    location: Location {
                        uri: Url {
                            scheme: "file",
                            cannot_be_a_base: false,
                            username: "",
                            password: None,
                            host: None,
                            port: None,
                            path: "/home/nmatsakis/versioned/dada/dada_tests/validate/duplicate_class_class.dada",
                            query: None,
                            fragment: None,
                        },
                        range: Range {
                            start: Position {
                                line: 2,
                                character: 7,
                            },
                            end: Position {
                                line: 2,
                                character: 10,
                            },
                        },
                    },
                    message: "ignoring this class for now",
                },
                DiagnosticRelatedInformation {
                    location: Location {
                        uri: Url {
                            scheme: "file",
                            cannot_be_a_base: false,
                            username: "",
                            password: None,
                            host: None,
                            port: None,
                            path: "/home/nmatsakis/versioned/dada/dada_tests/validate/duplicate_class_class.dada",
                            query: None,
                            fragment: None,
                        },
                        range: Range {
                            start: Position {
                                line: 1,
                                character: 7,
                            },
                            end: Position {
                                line: 1,
                                character: 10,
                            },
                        },
                    },
                    message: "the class is here",
                },
            ],
        ),
        tags: None,
    },
]