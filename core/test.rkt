; utilities useful for testing in other modules
#lang rosette

(provide (all-defined-out))

(require
  "animal.rkt"
  "tank.rkt"
  "../test.rkt")

(define (make-test-species #:type [typ 't] . args)
  (define (not-exists? pred) (not (ormap pred args)))
  (define id (fresh-symbol))
  (define extra-args
    (filter
      (λ (x) x)
      (list
        (and (not-exists? size?) (make-size 1))
        (and (not-exists? environment?) (environment warm-water 50))
        (and (not-exists? diet?) (food 'x 1)))))
  (apply make-fish `(,id ,typ ,@args ,@extra-args)))
