#lang rosette/safe

(provide (all-defined-out))

(require "../core.rkt")

(struct game-data
  (species
   tanks
   localization)
  #:transparent) 

(define (species-ref data id)
  (findf
    (λ (a) (equal? id (species-id a)))
    (game-data-species data)))
