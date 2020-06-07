; utilities useful for testing in other modules
#lang rosette

(provide (all-defined-out))

(require
  "animal.rkt"
  "tank.rkt"
  "../test.rkt")

(define (make-test-species #:id [id #f] #:type [typ #f] . args)
  (define (not-exists? pred) (not (ormap pred args)))
  (set! id (or id (fresh-symbol)))
  (set! typ (or typ (fresh-symbol)))
  (define extra-args
    (filter
      (λ (x) x)
      (list
        (and (not-exists? size?) (make-size 1))
        (and (not-exists? environment?) (environment warm-water 50))
        (and (not-exists? diet?) (food 'x 1)))))
  (apply make-fish `(,id ,typ ,@args ,@extra-args)))
