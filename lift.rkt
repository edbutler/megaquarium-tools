#lang rosette/safe

(provide (all-defined-out))

(require
  (for-syntax (only-in racket define-syntax-rule)))

(define (landmap f lst)
  (apply && (map f lst)))
(define (lormap f lst)
  (apply || (map f lst)))
(define (sum f lst)
  (apply + (map f lst)))
(define (max-by f lst)
  (apply max (map f lst)))

; shorthand for (if pred e empty)
(define-syntax-rule (maybe-list pred e)
  (if pred e '()))
; if pred, returns singleton list (list e) else empty
(define-syntax-rule (maybe-singleton pred e)
  (if pred (list e) '()))
; if (not pred), returns singleton list (list e) else empty
(define-syntax-rule (maybe-not-singleton pred e)
  (if pred '() (list e)))

