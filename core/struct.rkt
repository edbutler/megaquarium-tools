#lang racket

(require
  racket/provide-syntax
  (for-syntax
    racket/syntax
    racket/provide-syntax
    syntax/parse))

(begin-for-syntax
  (define symbol->keyword (compose1 string->keyword symbol->string))
  (define syntax->keyword (compose1 symbol->keyword syntax->datum))

  ; turns syntax of the form `((a ...) (b ...) ...)` into `(a ... b ... ...)`
  (define (flatten-syntax stx)
    (apply append (map syntax->list (syntax->list stx))))

  (define-syntax-class field
    (pattern
      id:id
      #:with kw-arg #`(#,(syntax->keyword #'id) id))
    ; maybe I don't actually want default exprs
    (pattern
      [id:id default-expr:expr]
      #:with kw-arg #`(#,(syntax->keyword #'id) [id default-expr]))))

(define-syntax (struct/kw stx)
  (syntax-parse stx
   [(_ id:id (field:field ...) struct-option ...)
    (with-syntax ([ctor (format-id #'id "make-~a" #'id)])
      #`(begin
        (struct id (field.id ...) #:transparent struct-option ...)
        (define (ctor #,@(flatten-syntax #'(field.kw-arg ...)))
          (id field.id ...))
      ))]))
  
(define-provide-syntax struct/kw-contract-out
  (syntax-parser
   [(_ id:id)
    (with-syntax ([ctor (format-id #'id "make-~a" #'id)])
      #'(combine-out
        (struct-out id)
        ctor))]))

(provide struct/kw)

(module+ main
  (expand-once #'(struct/kw foo (bar [baz 25])))
  (struct/kw foo (bar [baz 25]))
  (provide (struct/kw-contract-out foo))
)