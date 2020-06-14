#lang racket

(provide check-aquarium)

(require
  "data.rkt"
  "core.rkt"
  "lift.rkt"
  "constraint.rkt")

(define (do-check-aquarium)
  (void))
  ;(define aqm (read-aquarium (read-game-data) (current-input-port)))
  ;(displayln aqm))

(define (check-aquarium program-name args)
  (command-line
    #:program program-name 
    #:argv args
    #:usage-help
        "Checks the viability of the given aquarium."
        "Reads from the given file, or stdin if no file given"
    #:args ([filename #f])
    (if filename
      (with-input-from-file filename do-check-aquarium)
      (do-check-aquarium))))
