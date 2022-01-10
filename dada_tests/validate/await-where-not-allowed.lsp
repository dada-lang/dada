[
    Diagnostic {
        range: Range {
            start: Position {
                line: 15,
                character: 31,
            },
            end: Position {
                line: 15,
                character: 36,
            },
        },
        severity: Some(
            Error,
        ),
        code: None,
        source: None,
        message: "await is not permitted inside atomic sections",
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
                            path: "(local-file-prefix)/dada_tests/validate/await-where-not-allowed.dada",
                            query: None,
                            fragment: None,
                        },
                        range: Range {
                            start: Position {
                                line: 15,
                                character: 31,
                            },
                            end: Position {
                                line: 15,
                                character: 36,
                            },
                        },
                    },
                    message: "await is here",
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
                            path: "(local-file-prefix)/dada_tests/validate/await-where-not-allowed.dada",
                            query: None,
                            fragment: None,
                        },
                        range: Range {
                            start: Position {
                                line: 14,
                                character: 5,
                            },
                            end: Position {
                                line: 14,
                                character: 11,
                            },
                        },
                    },
                    message: "atomic section entered here",
                },
            ],
        ),
        tags: None,
    },
    Diagnostic {
        range: Range {
            start: Position {
                line: 8,
                character: 31,
            },
            end: Position {
                line: 8,
                character: 36,
            },
        },
        severity: Some(
            Error,
        ),
        code: None,
        source: None,
        message: "await is not permitted inside atomic sections",
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
                            path: "(local-file-prefix)/dada_tests/validate/await-where-not-allowed.dada",
                            query: None,
                            fragment: None,
                        },
                        range: Range {
                            start: Position {
                                line: 8,
                                character: 31,
                            },
                            end: Position {
                                line: 8,
                                character: 36,
                            },
                        },
                    },
                    message: "await is here",
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
                            path: "(local-file-prefix)/dada_tests/validate/await-where-not-allowed.dada",
                            query: None,
                            fragment: None,
                        },
                        range: Range {
                            start: Position {
                                line: 7,
                                character: 5,
                            },
                            end: Position {
                                line: 7,
                                character: 11,
                            },
                        },
                    },
                    message: "atomic section entered here",
                },
            ],
        ),
        tags: None,
    },
    Diagnostic {
        range: Range {
            start: Position {
                line: 2,
                character: 27,
            },
            end: Position {
                line: 2,
                character: 32,
            },
        },
        severity: Some(
            Error,
        ),
        code: None,
        source: None,
        message: "await is not permitted outside of async functions",
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
                            path: "(local-file-prefix)/dada_tests/validate/await-where-not-allowed.dada",
                            query: None,
                            fragment: None,
                        },
                        range: Range {
                            start: Position {
                                line: 2,
                                character: 27,
                            },
                            end: Position {
                                line: 2,
                                character: 32,
                            },
                        },
                    },
                    message: "await is here",
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
                            path: "(local-file-prefix)/dada_tests/validate/await-where-not-allowed.dada",
                            query: None,
                            fragment: None,
                        },
                        range: Range {
                            start: Position {
                                line: 1,
                                character: 1,
                            },
                            end: Position {
                                line: 1,
                                character: 4,
                            },
                        },
                    },
                    message: "fn not declared `async`",
                },
            ],
        ),
        tags: None,
    },
]