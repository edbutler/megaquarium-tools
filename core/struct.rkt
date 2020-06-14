#lang racket

(require (for-syntax syntax/parse racket/syntax))

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
        (struct id (field.id ...) struct-option ...)
        (define (ctor #,@(flatten-syntax #'(field.kw-arg ...)))
          (id field.id ...))
      ))]))
  
(provide struct/kw)

(define (test x) (add1 x))

(module+ main
  (expand-once #'(struct/kw foo (bar [baz 25]))))